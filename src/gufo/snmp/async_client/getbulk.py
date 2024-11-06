# ---------------------------------------------------------------------
# Gufo SNMP: GetBulkIter
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetBulkIter iterator."""

# Python modules
from typing import List, Optional, Tuple

# Gufo Labs Modules
from .._fast import GetIter as _Iter
from ..policer import BasePolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType
from .util import send_and_recv


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
        sock: SnmpClientSocketProtocol,
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

        def sender() -> None:
            self._sock.send_get_bulk(self._ctx)

        def receiver() -> List[Tuple[str, ValueType]]:
            return self._sock.recv_get_bulk(self._ctx)

        # Return item from buffer, if present
        if self._buffer:
            return self._buffer.pop(0)
        # Complete
        if self._stop:
            raise StopAsyncIteration
        self._buffer = await send_and_recv(
            self._fd, sender, receiver, self._policer, self._timeout
        )
        # End?
        if not self._buffer:
            raise StopAsyncIteration  # End of view
        self._stop = len(self._buffer) < self._max_repetitions
        # Having at least one item
        return self._buffer.pop(0)
