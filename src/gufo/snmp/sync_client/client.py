# ---------------------------------------------------------------------
# Gufo SNMP: SyncSnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""SyncSnmpSession implementation."""

# Python modules
from types import TracebackType
from typing import Dict, Iterable, Iterator, Optional, Tuple, Type, Union

# Gufo Labs modules
from .._fast import (
    SnmpV1ClientSocket,
    SnmpV2cClientSocket,
    SnmpV3ClientSocket,
)
from ..policer import BasePolicer, RPSPolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType
from ..user import User
from ..version import SnmpVersion
from .getbulk import GetBulkIter
from .getnext import GetNextIter

NS = 1_000_000_000.0


class SnmpSession(object):
    """
    Synchronous SNMP client session.

    Should be used either directly or via context manager.

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
        timeout_ns = int(timeout * NS)
        if version == SnmpVersion.v1:
            self._sock = SnmpV1ClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
                timeout_ns,
            )
        elif version == SnmpVersion.v2c:
            self._sock = SnmpV2cClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
                timeout_ns,
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
                timeout_ns,
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

    def __enter__(self: "SnmpSession") -> "SnmpSession":
        """Asynchronous context manager entry."""
        self.refresh()
        return self

    def __exit__(
        self: "SnmpSession",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Asynchronous context manager exit."""

    def get(self: "SnmpSession", oid: str) -> ValueType:
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
        if self._policer:
            self._policer.wait_sync()
        try:
            return self._sock.get(oid)
        except BlockingIOError as e:
            raise TimeoutError from e

    def get_many(
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
        if self._policer:
            self._policer.wait_sync()
        try:
            return self._sock.get_many(list(oids))
        except BlockingIOError as e:
            raise TimeoutError from e

    def getnext(
        self: "SnmpSession", oid: str
    ) -> Iterator[Tuple[str, ValueType]]:
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
        return GetNextIter(self._sock, oid, self._policer)

    def getbulk(
        self: "SnmpSession", oid: str, max_repetitions: Optional[int] = None
    ) -> Iterator[Tuple[str, ValueType]]:
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
            self._sock,
            oid,
            max_repetitions or self._max_repetitions,
            self._policer,
        )

    def fetch(
        self: "SnmpSession", oid: str
    ) -> Iterator[Tuple[str, ValueType]]:
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

    def refresh(self: "SnmpSession") -> None:
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
            self._sock.refresh()
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
        self._sock.refresh()

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
