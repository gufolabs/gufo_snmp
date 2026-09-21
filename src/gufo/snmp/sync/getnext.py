# ---------------------------------------------------------------------
# Gufo SNMP: GetNextIter
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""GetNextIter iterator."""

# Gufo Labs Modules
from .._fast import GetIter as _Iter
from ..policer import BasePolicer
from ..protocol import SnmpClientSocketProtocol
from ..typing import ValueType


class GetNextIter:
    """Wrap the series of the GetNext requests.

    Args:
        sock: Requsting SnmpClientSocket instance.
        oid: Base oid.
        policer: Optional BasePolicer instance to limit
            outgoing requests.
    """

    def __init__(
        self,
        sock: SnmpClientSocketProtocol,
        oid: str,
        policer: BasePolicer | None = None,
    ) -> None:
        self._sock = sock
        self._ctx = _Iter(oid)
        self._policer = policer

    def __iter__(self) -> "GetNextIter":
        """Return iterator."""
        return self

    def __next__(self) -> tuple[str, ValueType]:
        """Get next value."""
        if self._policer:
            self._policer.wait_sync()
        try:
            return self._sock.get_next(self._ctx)
        except StopAsyncIteration as e:
            raise StopIteration from e
        except BlockingIOError as e:
            raise TimeoutError from e
