# ---------------------------------------------------------------------
# Gufo SNMP: Socket protocol definition
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Dict, List, Protocol, Tuple, Union

# Gufo Labs modules
from ._fast import GetIter
from .typing import ValueType


class SnmpClientSocketProtocol(Protocol):
    def get_fd(self: "SnmpClientSocketProtocol") -> int: ...

    # .get()
    def get(self: "SnmpClientSocketProtocol", oid: str) -> ValueType: ...

    def send_get(self: "SnmpClientSocketProtocol", oid: str) -> None: ...

    def recv_get(self: "SnmpClientSocketProtocol") -> ValueType: ...

    # .get_many()
    def get_many(
        self: "SnmpClientSocketProtocol", oids: List[str]
    ) -> Dict[str, ValueType]: ...

    def send_get_many(
        self: "SnmpClientSocketProtocol", oids: List[str]
    ) -> None: ...

    def recv_get_many(
        self: "SnmpClientSocketProtocol",
    ) -> Dict[str, ValueType]: ...

    # .get_next
    def get_next(
        self: "SnmpClientSocketProtocol", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...

    def send_get_next(
        self: "SnmpClientSocketProtocol", iter_getnext: GetIter
    ) -> None: ...

    def recv_get_next(
        self: "SnmpClientSocketProtocol", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...

    # .get_bulk
    def get_bulk(
        self: "SnmpClientSocketProtocol", iter_getbulk: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...

    def send_get_bulk(
        self: "SnmpClientSocketProtocol", iter_getbulk: GetIter
    ) -> None: ...

    def recv_get_bulk(
        self: "SnmpClientSocketProtocol", iter_getnext: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...
