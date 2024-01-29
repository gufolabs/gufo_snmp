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
from .._fast import GetNextIter as _Iter
from ..policer import BasePolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType


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
        sock: SnmpClientSocketProtocol,
        oid: str,
        timeout: float,
        policer: Optional[BasePolicer] = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid)
        self._fd = sock.get_fd()
        self._timeout = timeout
        self._policer = policer

    def __iter__(self: "GetNextIter") -> "GetNextIter":
        """Return iterator."""
        return self

    def __next__(self: "GetNextIter") -> Tuple[str, ValueType]:
        """Get next value."""
        try:
            return self._sock.sync_getnext(self._ctx)
        except StopAsyncIteration as e:
            raise StopIteration from e
