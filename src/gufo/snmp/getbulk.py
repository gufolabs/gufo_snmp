# ---------------------------------------------------------------------
# Gufo SNMP: GetBulkIter
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetBulkIter iterator."""


# Python modules
import asyncio
from typing import List, Tuple

# Gufo Labs Modules
from ._fast import GetBulkIter as _Iter
from ._fast import SnmpClientSocket
from .typing import ValueType


class GetBulkIter(object):
    """Wrap the series of the GetBulk requests.

    Args:
        oid: Base oid.
        timeout: Request timeout.
        max_repetitions: Max amount of iterms per response.
    """

    def __init__(
        self: "GetBulkIter",
        sock: SnmpClientSocket,
        oid: str,
        timeout: float,
        max_repetitions: int,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid, max_repetitions)
        self._fd = sock.get_fd()
        self._timeout = timeout
        self._max_repetitions = max_repetitions
        self._buffer: List[Tuple[str, ValueType]] = []
        self._stop = False

    def __aiter__(self: "GetBulkIter") -> "GetBulkIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetBulkIter") -> Tuple[str, ValueType]:
        """Get next value."""

        async def get_response() -> List[Tuple[str, ValueType]]:
            while True:
                r_ev = asyncio.Event()
                # Wait until data will be available
                loop.add_reader(self._fd, r_ev.set)
                await r_ev.wait()
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
        # Wait for socket became writable
        w_ev = asyncio.Event()
        loop.add_writer(self._fd, w_ev.set)
        await w_ev.wait()
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
