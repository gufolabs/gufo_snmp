# ---------------------------------------------------------------------
# Gufo SNMP: GetNextIter
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetNextIter iterator."""

# Python modules
from typing import Optional, Tuple

# Gufo Labs Modules
from .._fast import GetIter as _Iter
from ..policer import BasePolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType


class GetNextIter(object):
    """Wrap the series of the GetNext requests.

    Args:
        sock: Requsting SnmpClientSocket instance.
        oid: Base oid.
        policer: Optional BasePolicer instance to limit
            outgoing requests.
    """

    def __init__(
        self: "GetNextIter",
        sock: SnmpClientSocketProtocol,
        oid: str,
        policer: Optional[BasePolicer] = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid)
        self._policer = policer

    def __iter__(self: "GetNextIter") -> "GetNextIter":
        """Return iterator."""
        return self

    def __next__(self: "GetNextIter") -> Tuple[str, ValueType]:
        """Get next value."""
        if self._policer:
            self._policer.wait_sync()
        try:
            return self._sock.get_next(self._ctx)
        except StopAsyncIteration as e:
            raise StopIteration from e
        except BlockingIOError as e:
            raise TimeoutError from e
