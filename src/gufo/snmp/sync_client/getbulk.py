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


class GetBulkIter(object):
    """Wrap the series of the GetBulk requests.

    Args:
        sock: Parent SnmpClientSocket.
        oid: Base oid.
        max_repetitions: Max amount of iterms per response.
        policer: Optional BasePolicer instance to limit requests.
    """

    def __init__(
        self: "GetBulkIter",
        sock: SnmpClientSocketProtocol,
        oid: str,
        max_repetitions: int,
        policer: Optional[BasePolicer] = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid, max_repetitions)
        self._max_repetitions = max_repetitions
        self._buffer: List[Tuple[str, ValueType]] = []
        self._stop = False
        self._policer = policer

    def __iter__(self: "GetBulkIter") -> "GetBulkIter":
        """Return asynchronous iterator."""
        return self

    def __next__(self: "GetBulkIter") -> Tuple[str, ValueType]:
        """Get next value."""
        # Return item from buffer, if present
        if self._buffer:
            return self._buffer.pop(0)
        # Complete
        if self._stop:
            raise StopIteration
        # Policer
        if self._policer:
            self._policer.wait_sync()
        try:
            self._buffer = self._sock.get_bulk(self._ctx)
        except BlockingIOError as e:
            raise TimeoutError from e
        except StopAsyncIteration as e:
            raise StopIteration from e
        # End
        if not self._buffer:
            raise StopIteration  # End of view
        self._stop = len(self._buffer) < self._max_repetitions
        # Having at least one item
        return self._buffer.pop(0)
