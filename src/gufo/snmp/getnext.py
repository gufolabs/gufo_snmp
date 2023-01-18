# ---------------------------------------------------------------------
# Gufo SNMP: SnmpSession
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Any, Tuple
import asyncio

# Gufo Labs Modules
from ._fast import GetNextIter as _Iter, SnmpClientSocket


class GetNextIter(object):
    def __init__(self, sock: SnmpClientSocket, oid: str, timeout: float):
        self._sock = sock
        self._ctx = _Iter(oid)
        self._fd = sock.get_fd()
        self._timeout = timeout

    def __aiter__(self) -> "GetNextIter":
        return self

    async def __anext__(self) -> Tuple[str, Any]:
        async def get_response() -> Tuple[str, Any]:
            while True:
                r_ev = asyncio.Event()
                # Wait until data will be available
                loop.add_reader(self._fd, r_ev.set)
                await r_ev.wait()
                # Read data or get BlockingIOError
                # if no valid data available.
                try:
                    return self._sock.recv_getresponse_next(self._ctx)
                except BlockingIOError:
                    continue

        loop = asyncio.get_running_loop()
        # Wait for socket became writable
        w_ev = asyncio.Event()
        loop.add_writer(self._fd, w_ev.set)
        await w_ev.wait()
        loop.remove_writer(self._fd)
        # Send request
        self._sock.send_getnext(self._ctx)
        # Await response or timeout
        return await asyncio.wait_for(get_response(), self._timeout)
