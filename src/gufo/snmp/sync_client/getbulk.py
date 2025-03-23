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
        self._buffer: List[Union[Tuple[str, ValueType], None]] = []
        self._policer = policer

    def __iter__(self: "GetBulkIter") -> "GetBulkIter":
        """Return asynchronous iterator."""
        return self

    def __next__(self: "GetBulkIter") -> Tuple[str, ValueType]:
        """Get next value."""

        def pop_or_stop() -> Tuple[str, ValueType]:
            v = self._buffer.pop(0)
            if v is None:
                raise StopIteration
            return v

        # Return item from buffer, if present
        if self._buffer:
            return pop_or_stop()
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
        return pop_or_stop()
