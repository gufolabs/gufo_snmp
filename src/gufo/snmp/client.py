# ---------------------------------------------------------------------
# Gufo SNMP: SnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import enum
import asyncio
from typing import Any, Iterable, Dict

# Gufo Labs modules
from ._fast import SnmpClientSocket


class SnmpVersion(enum.IntEnum):
    v1 = 0
    v2c = 1


class SnmpSession(object):
    """
    Client SNMP session. Should be used either directly
    or via asynchronous context manager.

    Args:
        addr: SNMP agent address, eigher IPv4 or IPv6.
        port: SNMP agent port.
        community: SNMP community.
        version: Protocol version.
        timeout: Request timeout in seconds.
        tos: Set ToS/DSCP mark on egress packets.
        send_buffer: Send buffer size for UDP socket.
            0 - use default size.
        recv_buffer: Receive buffer size for UDP socket.
            0 - use default size.

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
        self,
        addr: str,
        port: int = 161,
        community: str = "public",
        version: SnmpVersion = SnmpVersion.v2c,
        timeout: float = 10.0,
        tos: int = 0,
        send_buffer: int = 0,
        recv_buffer: int = 0,
        # timeout
    ):
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

    async def __aenter__(self) -> "SnmpSession":
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass

    async def get(self, oid: str) -> Any:
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
            SNMPError: On other SNMP-related errors.
        """

        async def get_response() -> Any:
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
        return await asyncio.wait_for(get_response(), self._timeout)

    async def get_many(self, oids: Iterable[str]) -> Dict[str, Any]:
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
            SNMPError: On other SNMP-related errors.
        """

        async def get_response() -> Dict[str, Any]:
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
        return await asyncio.wait_for(get_response(), self._timeout)
