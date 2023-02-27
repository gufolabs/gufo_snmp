# ---------------------------------------------------------------------
# Gufo SNMP: GetNextIter
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetNextIter iterator."""


# Python modules
import asyncio
from typing import Optional, Tuple

# Gufo Labs Modules
from ._fast import GetNextIter as _Iter
from ._fast import SnmpClientSocket
from .policer import BasePolicer
from .typing import ValueType


class GetNextIter(object):
    """Wrap the series of the GetNext requests.

    Args:
        sock: Requsting SnmpClientSocket instance.
        oid: Base oid.
        timeout: Request timeout.
        policer: Optional BasePolicer instance to limit
            outgoing requests.
    """

    def __init__(
        self: "GetNextIter",
        sock: SnmpClientSocket,
        oid: str,
        timeout: float,
        policer: Optional[BasePolicer] = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid)
        self._fd = sock.get_fd()
        self._timeout = timeout
        self._policer = policer

    def __aiter__(self: "GetNextIter") -> "GetNextIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetNextIter") -> Tuple[str, ValueType]:
        """Get next value."""

        async def get_response() -> Tuple[str, ValueType]:
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
                    return self._sock.recv_getresponse_next(self._ctx)
                except BlockingIOError:
                    continue

        loop = asyncio.get_running_loop()
        # Process limits
        if self._policer:
            await self._policer.wait()
        # Wait for socket became writable
        w_ev = asyncio.Event()
        loop.add_writer(self._fd, w_ev.set)
        try:
            await w_ev.wait()
        finally:
            loop.remove_writer(self._fd)
        # Send request
        self._sock.send_getnext(self._ctx)
        # Await response or timeout
        try:
            return await asyncio.wait_for(get_response(), self._timeout)
        except asyncio.TimeoutError as e:
            raise TimeoutError from e  # Remap the error
