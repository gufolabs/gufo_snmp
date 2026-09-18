# ---------------------------------------------------------------------
# Gufo SNMP: Socket protocol definition
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Protocol

# Gufo Labs modules
from ._fast import GetIter
from .typing import ValueType


class SnmpClientSocketProtocol(Protocol):
    def get_fd(self) -> int: ...

    # .get()
    def get(self, oid: str) -> ValueType: ...

    def send_get(self, oid: str) -> None: ...

    def recv_get(self) -> ValueType: ...

    # .get_many()
    def get_many(self, oids: list[str]) -> dict[str, ValueType]: ...

    def send_get_many(self, oids: list[str]) -> None: ...

    def recv_get_many(
        self,
    ) -> dict[str, ValueType]: ...

    # .get_next
    def get_next(self, iter_getnext: GetIter) -> tuple[str, ValueType]: ...

    def send_get_next(self, iter_getnext: GetIter) -> None: ...

    def recv_get_next(
        self, iter_getnext: GetIter
    ) -> tuple[str, ValueType]: ...

    # .get_bulk
    def get_bulk(
        self, iter_getbulk: GetIter
    ) -> list[tuple[str, ValueType] | None]: ...

    def send_get_bulk(self, iter_getbulk: GetIter) -> None: ...

    def recv_get_bulk(
        self, iter_getnext: GetIter
    ) -> list[tuple[str, ValueType] | None]: ...
