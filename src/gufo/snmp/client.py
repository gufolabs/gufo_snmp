# ---------------------------------------------------------------------
# Gufo SNMP: SnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""SnmpSession implementation."""

# Python modules
from types import TracebackType
from typing import AsyncIterator, Dict, Iterable, Optional, Tuple, Type, Union

# Gufo Labs modules
from ._fast import (
    SnmpV1ClientSocket,
    SnmpV2cClientSocket,
)
from .getbulk import GetBulkIter
from .getnext import GetNextIter
from .policer import BasePolicer, RPSPolicer
from .protocol import SnmpClientSocketProtocol
from .typing import ValueType
from .util import send_and_recv
from .version import SnmpVersion


class SnmpSession(object):
    """
    SNMP client session.

    Should be used either directly or via asynchronous context manager.

    Args:
        addr: SNMP agent address, either IPv4 or IPv6.
        port: SNMP agent port.
        community: SNMP community.
        version: Protocol version.
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
        version: SnmpVersion = SnmpVersion.v2c,
        timeout: float = 10.0,
        tos: int = 0,
        send_buffer: int = 0,
        recv_buffer: int = 0,
        max_repetitions: int = 20,
        allow_bulk: bool = True,
        policer: Optional[BasePolicer] = None,
        limit_rps: Optional[Union[int, float]] = None,
    ) -> None:
        self._sock: SnmpClientSocketProtocol
        if version == SnmpVersion.v1:
            self._sock = SnmpV1ClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
            )
        elif version == SnmpVersion.v2c:
            self._sock = SnmpV2cClientSocket(
                f"{addr}:{port}",
                community,
                tos,
                send_buffer,
                recv_buffer,
            )
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
        return self

    async def __aexit__(
        self: "SnmpSession",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Asynchronous context manager exit."""

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

        return await send_and_recv(
            self._fd,
            sender,
            self._sock.recv_getresponse,
            self._policer,
            self._timeout,
        )

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

        return await send_and_recv(
            self._fd,
            sender,
            self._sock.recv_getresponse_many,
            self._policer,
            self._timeout,
        )

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
        return GetNextIter(self._sock, oid, self._timeout, self._policer)

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
            self._sock,
            oid,
            self._timeout,
            max_repetitions or self._max_repetitions,
            self._policer,
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
