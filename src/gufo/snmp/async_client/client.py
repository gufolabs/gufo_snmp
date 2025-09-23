# ---------------------------------------------------------------------
# Gufo SNMP: Async SnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""SnmpSession implementation."""

# Python modules
from asyncio import Future, get_running_loop, wait_for
from asyncio import TimeoutError as AIOTimeoutError
from types import TracebackType
from typing import (
    AsyncIterator,
    Callable,
    Dict,
    Iterable,
    List,
    Optional,
    Tuple,
    Type,
    TypeVar,
    Union,
)

# Gufo Labs modules
from .._fast import (
    GetIter,
    SnmpV1ClientSocket,
    SnmpV2cClientSocket,
    SnmpV3ClientSocket,
)
from ..policer import BasePolicer, RPSPolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType
from ..user import User
from ..version import SnmpVersion

T = TypeVar("T")


class SnmpSession(object):
    """
    SNMP client session.

    Should be used either directly or via asynchronous context manager.

    Args:
        addr: SNMP agent address, either IPv4 or IPv6.
        port: SNMP agent port.
        community: SNMP community (v1, v2c).
        engine_id: SNMP Engine id (v3).
        user: User instance (v3).
        version: Protocol version. Autodetect if omitted:

            * v3: if `user` is set.
            * v2c: otherwise.

        timeout: Request timeout in seconds.
        tos: Set ToS/DSCP mark on egress packets.
        send_buffer: Send buffer size for UDP socket.
            0 - use default size.
        recv_buffer: Receive buffer size for UDP socket.
            0 - use default size.
        max_repetitions: Default max_repetitions for getbulk.
        allow_bulk: Allow using GETBULK in SnmpSession.fetch()
            whenever possible.
        policer: Optional `BasePolicer` instance to limit
            outgoing requests. Overrides `limit_rps` parameter.
        limit_rps: Limit outgouing requests to `limit_rps`
            requests per second.

    Example:
        ``` py
        session = SnmpSession("127.0.0.1")
        r = await session.get("1.3.6.1.2.1.1.6.0")
        ```

    Example:
        ``` py
        async with SnmpSession("127.0.0.1") as session:
            r = await session.get("1.3.6.1.2.1.1.6.0")
        ```
    """

    def __init__(
        self: "SnmpSession",
        addr: str,
        port: int = 161,
        community: str = "public",
        engine_id: Optional[bytes] = None,
        user: Optional[User] = None,
        version: Optional[SnmpVersion] = None,
        timeout: float = 10.0,
        tos: int = 0,
        send_buffer: int = 0,
        recv_buffer: int = 0,
        max_repetitions: int = 20,
        allow_bulk: bool = True,
        policer: Optional[BasePolicer] = None,
        limit_rps: Optional[Union[int, float]] = None,
    ) -> None:
        # Detect version
        if version is None:
            version = SnmpVersion.v2c if user is None else SnmpVersion.v3
        self._sock: SnmpClientSocketProtocol
        self._to_refresh = False
        self._deferred_user: Optional[User] = None
        if version == SnmpVersion.v1:
            self._sock = SnmpV1ClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
                0,
            )
        elif version == SnmpVersion.v2c:
            self._sock = SnmpV2cClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
                0,
            )
        elif version == SnmpVersion.v3:
            if not user:
                msg = "SNMPv3 requires user"
                raise ValueError(msg)
            if not engine_id:
                # Defer authentication until engine id is discovered
                self._deferred_user = user
                user = User.default()
            self._sock = SnmpV3ClientSocket(
                f"{addr}:{port}",
                engine_id if engine_id else b"",
                user.name,
                user.get_auth_alg(),
                user.get_auth_key(),
                user.get_priv_alg(),
                user.get_priv_key(),
                tos,
                send_buffer,
                recv_buffer,
                0,
            )
            self._to_refresh = not engine_id or user.require_auth()
        else:
            msg = "Invalid SNMP Protocol"
            raise ValueError(msg)
        self._fd = self._sock.get_fd()
        self._timeout = timeout
        self._max_repetitions = max_repetitions
        if version == SnmpVersion.v1:
            self._allow_bulk = False
        else:
            self._allow_bulk = allow_bulk
        self._policer: Optional[BasePolicer] = None
        if policer:
            self._policer = policer
        elif limit_rps:
            self._policer = RPSPolicer(float(limit_rps))

    async def __aenter__(self: "SnmpSession") -> "SnmpSession":
        """Asynchronous context manager entry."""
        await self.refresh()
        return self

    async def __aexit__(
        self: "SnmpSession",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Asynchronous context manager exit."""

    async def _send(self, sender: Callable[[], None]) -> None:
        """
        Execute callable when socket in writable.

        Args:
            sender: Function to execute a write operation.
        """

        def callback() -> None:
            try:
                sender()
                future.set_result(None)
            except BaseException as e:  # noqa: BLE001
                future.set_exception(e)

        if self._policer:
            await self._policer.wait()
        try:
            # Hot path.
            # Usually successful unless output buffer is full
            sender()
        except BlockingIOError:
            # Buffer is full, wait
            loop = get_running_loop()
            future = loop.create_future()
            loop.add_writer(self._fd, callback)
            try:
                await future
            finally:
                loop.remove_writer(self._fd)

    async def _recv(self, receiver: Callable[[], T]) -> T:
        """
        Execute callable when socket is ready.

        Args:
            receiver: Function to execute read operation.
        """

        def on_readable() -> None:
            if not fut.done():
                fut.set_result(None)

        loop = get_running_loop()
        while True:
            fut: Future[None] = loop.create_future()
            loop.add_reader(self._fd, on_readable)
            try:
                await wait_for(fut, self._timeout)
                return receiver()
            except BlockingIOError:
                continue
            except AIOTimeoutError as e:
                raise TimeoutError from e  # Remap the error
            finally:
                loop.remove_reader(self._fd)

    async def get(self: "SnmpSession", oid: str) -> ValueType:
        """
        Send SNMP GET request and await for response.

        Args:
            oid: OID in numeric format, no leading dot.

        Returns:
            Request result. Return type depends on requested oid.

        Raises:
            ValueError: On invalid oid format.
            OSError: When unable to send request.
            TimeoutError: When timed out.
            NoSuchInstance: When requested key is not found.
            SnmpError: On other SNMP-related errors.
        """

        def sender() -> None:
            self._sock.send_get(oid)

        await self._send(sender)
        return await self._recv(self._sock.recv_get)

    async def get_many(
        self: "SnmpSession", oids: Iterable[str]
    ) -> Dict[str, ValueType]:
        """
        Send SNMP GET request for multiple oids and await for response.

        Args:
            oids: Iterable of oids in numeric format, no leading dots.

        Returns:
            Dict where keys are requested oids, values are returned values.
            Types of values are depends on requested oids.

        Note:
            There is no guarante that all requested oids are present in
            result dict. Some values may be missed if not returned by agent.

        Raises:
            ValueError: On invalid oid format.
            OSError: When unable to send request.
            TimeoutError: When timed out.
            RuntimeError: On Python runtime failure.
            SnmpError: On other SNMP-related errors.
        """

        def sender() -> None:
            self._sock.send_get_many(list(oids))

        await self._send(sender)
        return await self._recv(self._sock.recv_get_many)

    def getnext(
        self: "SnmpSession", oid: str
    ) -> AsyncIterator[Tuple[str, ValueType]]:
        """
        Iterate over oids.

        Args:
            oid: Starting oid

        Returns:
            Asynchronous iterator yielding pair of (oid, value)

        Example:
            ``` py
            async for oid, value in session.getnext("1.3.6"):
                print(oid, value)
            ```
        """
        return GetNextIter(self, oid)

    def getbulk(
        self: "SnmpSession", oid: str, max_repetitions: Optional[int] = None
    ) -> AsyncIterator[Tuple[str, ValueType]]:
        """
        Iterate over oids.

        Args:
            oid: Starting oid
            max_repetitions: Maximal amount of items per response.
                Override the SnmpSession's defaults.

        Returns:
            Asynchronous iterator yielding pair of (oid, value)

        Example:
            ``` py
            async for oid, value in session.getbulk("1.3.6"):
                print(oid, value)
            ```
        """
        return GetBulkIter(
            self,
            oid,
            max_repetitions or self._max_repetitions,
        )

    def fetch(
        self: "SnmpSession", oid: str
    ) -> AsyncIterator[Tuple[str, ValueType]]:
        """
        Iterate over oids using fastest method available.

        When SnmpSession's `allow_bulk` is set, use
        `SnmpSession.getbulk()` on SNMPv2, otherwise
        use `SnmpSession.getnext()`.

        Args:
            oid: Starting oid

        Returns:
            Asynchronous iterator yielding pair of (oid, value)

        Example:
            ``` py
            async for oid, value in session.fetch("1.3.6"):
                print(oid, value)
            ```
        """
        if self._allow_bulk:
            return self.getbulk(oid)
        return self.getnext(oid)

    async def refresh(self: "SnmpSession") -> None:
        """
        Send and receive REPORT to refresh authentication state.

        SNMPv3 only.

        Refresh sent automatically on entering
        the SnmpSession and should be resent manually
        if over 150 seconds left from the last request.
        """
        if (
            not isinstance(self._sock, SnmpV3ClientSocket)
            or not self._to_refresh
        ):
            return

        if self._deferred_user:
            # First check runs engine id discovery
            await self._send(self._sock.send_refresh)
            await self._recv(self._sock.recv_refresh)
            # Set and localize actual keys
            self._sock.set_keys(
                self._deferred_user.name,
                self._deferred_user.get_auth_alg(),
                self._deferred_user.get_auth_key(),
                self._deferred_user.get_priv_alg(),
                self._deferred_user.get_priv_key(),
            )
            # Adjust refresh settings
            self._to_refresh = self._deferred_user.require_auth()
            # Forget deferred user
            self._deferred_user = None

        # Refresh engine boots and time
        await self._send(self._sock.send_refresh)
        await self._recv(self._sock.recv_refresh)

    def get_engine_id(self: "SnmpSession") -> bytes:
        """
        Get effective engine id.

        Returns:
            Engine id as bytes.
        """
        if not isinstance(self._sock, SnmpV3ClientSocket):
            msg = "Must use SNMPv3"
            raise NotImplementedError(msg)
        return self._sock.get_engine_id()


class GetNextIter(object):
    """Wrap the series of the GetNext requests.

    Args:
        session: Parent SnmpSession.
        oid: Base oid.
    """

    def __init__(
        self: "GetNextIter",
        session: SnmpSession,
        oid: str,
    ) -> None:
        self._session = session
        self._sock = session._sock
        self._ctx = GetIter(oid)

    def __aiter__(self: "GetNextIter") -> "GetNextIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetNextIter") -> Tuple[str, ValueType]:
        """Get next value."""

        def sender() -> None:
            self._sock.send_get_next(self._ctx)

        def receiver() -> Tuple[str, ValueType]:
            return self._sock.recv_get_next(self._ctx)

        await self._session._send(sender)
        return await self._session._recv(receiver)


class GetBulkIter(object):
    """Wrap the series of the GetBulk requests.

    Args:
        session: Parent SnmpSession.
        oid: Base oid.
        max_repetitions: Max amount of iterms per response.
    """

    def __init__(
        self: "GetBulkIter",
        session: SnmpSession,
        oid: str,
        max_repetitions: int,
    ) -> None:
        self._session = session
        self._sock = session._sock
        self._ctx = GetIter(oid, max_repetitions)
        self._max_repetitions = max_repetitions
        self._buffer: List[Union[Tuple[str, ValueType], None]] = []

    def __aiter__(self: "GetBulkIter") -> "GetBulkIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetBulkIter") -> Tuple[str, ValueType]:
        """Get next value."""

        def sender() -> None:
            self._sock.send_get_bulk(self._ctx)

        def receiver() -> List[Union[Tuple[str, ValueType], None]]:
            return self._sock.recv_get_bulk(self._ctx)

        def pop_or_stop() -> Tuple[str, ValueType]:
            v = self._buffer.pop(0)
            if v is None:
                raise StopAsyncIteration
            return v

        # Return item from buffer, if present
        if self._buffer:
            return pop_or_stop()
        await self._session._send(sender)
        self._buffer = await self._session._recv(receiver)
        # End?
        if not self._buffer:
            raise StopAsyncIteration  # End of view
        # Having at least one item
        return pop_or_stop()
