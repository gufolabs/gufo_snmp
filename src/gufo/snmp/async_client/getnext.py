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
from .util import send_and_recv


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

    def __aiter__(self: "GetNextIter") -> "GetNextIter":
        """Return asynchronous iterator."""
        return self

    async def __anext__(self: "GetNextIter") -> Tuple[str, ValueType]:
        """Get next value."""

        def sender() -> None:
            self._sock.send_get_next(self._ctx)

        def receiver() -> Tuple[str, ValueType]:
            return self._sock.recv_get_next(self._ctx)

        return await send_and_recv(
            self._fd, sender, receiver, self._policer, self._timeout
        )
