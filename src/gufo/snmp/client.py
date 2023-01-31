# ---------------------------------------------------------------------
# Gufo SNMP: SnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""SnmpSession implementation."""

# Python modules
import asyncio
from types import TracebackType
from typing import AsyncIterator, Dict, Iterable, Optional, Tuple, Type

# Gufo Labs modules
from ._fast import SnmpClientSocket
from .getbulk import GetBulkIter
from .getnext import GetNextIter
from .typing import ValueType
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
    ) -> None:
        self._sock = SnmpClientSocket(
            f"{addr}:{port}",
            community,
            version.value,
            tos,
            send_buffer,
            recv_buffer,
        )
        self._fd = self._sock.get_fd()
        self._timeout = timeout
        self._max_repetitions = max_repetitions
        if version == SnmpVersion.v1:
            self._allow_bulk = False
        else:
            self._allow_bulk = True

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

        async def get_response() -> ValueType:
            while True:
                r_ev = asyncio.Event()
                # Wait until data will be available
                loop.add_reader(self._fd, r_ev.set)
                await r_ev.wait()
                # Read data or get BlockingIOError
                # if no valid data available.
                try:
                    return self._sock.recv_getresponse()
                except BlockingIOError:
                    continue

        loop = asyncio.get_running_loop()
        # Wait for socket became writable
        w_ev = asyncio.Event()
        loop.add_writer(self._fd, w_ev.set)
        await w_ev.wait()
        loop.remove_writer(self._fd)
        # Send request
        self._sock.send_get(oid)
        # Await response or timeout
        try:
            return await asyncio.wait_for(get_response(), self._timeout)
        except asyncio.TimeoutError as e:
            raise TimeoutError from e  # Remap the error

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

        async def get_response() -> Dict[str, ValueType]:
            while True:
                r_ev = asyncio.Event()
                # Wait until data will be available
                loop.add_reader(self._fd, r_ev.set)
                await r_ev.wait()
                # Read data or get BlockingIOError
                # if no valid data available.
                try:
                    return self._sock.recv_getresponse_many()
                except BlockingIOError:
                    continue

        loop = asyncio.get_running_loop()
        # Wait for socket became writable
        w_ev = asyncio.Event()
        loop.add_writer(self._fd, w_ev.set)
        await w_ev.wait()
        loop.remove_writer(self._fd)
        # Send request
        self._sock.send_get_many(list(oids))
        # Await response or timeout
        try:
            return await asyncio.wait_for(get_response(), self._timeout)
        except asyncio.TimeoutError as e:
            raise TimeoutError from e  # Remap the error

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
        return GetNextIter(self._sock, oid, self._timeout)

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
