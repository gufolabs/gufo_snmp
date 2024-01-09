# ---------------------------------------------------------------------
# Gufo SNMP: Socket protocol definition
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Dict, List, Protocol, Tuple

# Gufo Labs modules
from ._fast import GetBulkIter, GetNextIter
from .typing import ValueType


class SnmpClientSocketProtocol(Protocol):
    def get_fd(self: "SnmpClientSocketProtocol") -> int:
        ...

    def send_get(self: "SnmpClientSocketProtocol", oid: str) -> None:
        ...

    def send_get_many(
        self: "SnmpClientSocketProtocol", oids: List[str]
    ) -> None:
        ...

    def send_getnext(
        self: "SnmpClientSocketProtocol", iter_getnext: GetNextIter
    ) -> None:
        ...

    def send_getbulk(
        self: "SnmpClientSocketProtocol", iter_getbulk: GetBulkIter
    ) -> None:
        ...

    def recv_getresponse(self: "SnmpClientSocketProtocol") -> ValueType:
        ...

    def recv_getresponse_many(
        self: "SnmpClientSocketProtocol",
    ) -> Dict[str, ValueType]:
        ...

    def recv_getresponse_next(
        self: "SnmpClientSocketProtocol", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]:
        ...

    def recv_getresponse_bulk(
        self: "SnmpClientSocketProtocol", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]:
        ...
