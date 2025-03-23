# ---------------------------------------------------------------------
# Gufo SNMP: GetBulkIter
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetBulkIter iterator."""

# Python modules
from typing import List, Optional, Tuple, Union

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
        self._buffer: List[Union[Tuple[str, ValueType], None]] = []
        self._policer = policer

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
        self._buffer = await send_and_recv(
            self._fd, sender, receiver, self._policer, self._timeout
        )
        # End?
        if not self._buffer:
            raise StopAsyncIteration  # End of view
        # Having at least one item
        return pop_or_stop()
