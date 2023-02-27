# ---------------------------------------------------------------------
# Gufo SNMP: GetBulkIter
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetBulkIter iterator."""


# Python modules
import asyncio
from typing import List, Optional, Tuple

# Gufo Labs Modules
from ._fast import GetBulkIter as _Iter
from ._fast import SnmpClientSocket
from .policer import BasePolicer
from .typing import ValueType


class GetBulkIter(object):
    """Wrap the series of the GetBulk requests.

    Args:
        sock: Parent SnmpClientSocket.
        oid: Base oid.
        timeout: Request timeout.
        max_repetitions: Max amount of iterms per response.
        policer: Optional BasePolicer instance to limit requests.
    """

    def __init__(
        self: "GetBulkIter",
        sock: SnmpClientSocket,
        oid: str,
        timeout: float,
        max_repetitions: int,
        policer: Optional[BasePolicer] = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid, max_repetitions)
        self._fd = sock.get_fd()
        self._timeout = timeout
        self._max_repetitions = max_repetitions
        self._buffer: List[Tuple[str, ValueType]] = []
        self._stop = False
        self._policer = policer

    def __aiter__(self: "GetBulkIter") -> "GetBulkIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetBulkIter") -> Tuple[str, ValueType]:
        """Get next value."""

        async def get_response() -> List[Tuple[str, ValueType]]:
            while True:
                # Wait until data will be available
                r_ev = asyncio.Event()
                loop.add_reader(self._fd, r_ev.set)
                try:
                    await r_ev.wait()
                finally:
                    loop.remove_reader(self._fd)
                # Read data or get BlockingIOError
                # if no valid data available.
                try:
                    return self._sock.recv_getresponse_bulk(self._ctx)
                except BlockingIOError:
                    continue

        # Return item from buffer, if present
        if self._buffer:
            return self._buffer.pop(0)
        if self._stop:
            raise StopAsyncIteration
        # Send request
        loop = asyncio.get_running_loop()
        # Process limits
        if self._policer:
            await self._policer.wait()
        r_ev = asyncio.Event()
        loop.add_writer(self._fd, r_ev.set)
        try:
            await r_ev.wait()
        finally:
            loop.remove_writer(self._fd)
        # Send request
        self._sock.send_getbulk(self._ctx)
        # Await response or timeout
        try:
            self._buffer = await asyncio.wait_for(
                get_response(), self._timeout
            )
        except asyncio.TimeoutError as e:
            raise TimeoutError from e  # Remap the error
        if not self._buffer:
            raise StopAsyncIteration  # End of view
        self._stop = len(self._buffer) < self._max_repetitions
        # Having at least one item
        return self._buffer.pop(0)
