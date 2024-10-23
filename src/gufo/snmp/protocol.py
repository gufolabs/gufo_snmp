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

    # .get()
    def get(self: "SnmpClientSocketProtocol", oid: str) -> ValueType:
        ...

    def send_get(self: "SnmpClientSocketProtocol", oid: str) -> None:
        ...

    def recv_get(self: "SnmpClientSocketProtocol") -> ValueType:
        ...

    # .get_many()
    def get_many(
        self: "SnmpClientSocketProtocol", oids: List[str]
    ) -> Dict[str, ValueType]:
        ...

    def send_get_many(
        self: "SnmpClientSocketProtocol", oids: List[str]
    ) -> None:
        ...

    def recv_get_many(
        self: "SnmpClientSocketProtocol",
    ) -> Dict[str, ValueType]:
        ...

    # Rest
    def send_getnext(
        self: "SnmpClientSocketProtocol", iter_getnext: GetNextIter
    ) -> None:
        ...

    def send_getbulk(
        self: "SnmpClientSocketProtocol", iter_getbulk: GetBulkIter
    ) -> None:
        ...

    def recv_getresponse_next(
        self: "SnmpClientSocketProtocol", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]:
        ...

    def recv_getresponse_bulk(
        self: "SnmpClientSocketProtocol", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]:
        ...

    def sync_getnext(
        self: "SnmpClientSocketProtocol", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]:
        ...

    def sync_getbulk(
        self: "SnmpClientSocketProtocol", iter_getbulk: GetBulkIter
    ) -> List[Tuple[str, ValueType]]:
        ...
