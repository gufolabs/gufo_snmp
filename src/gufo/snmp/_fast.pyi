# ---------------------------------------------------------------------
# Gufo SNMP: _fast typing
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Dict, List, Tuple

# Gufo Labs modules
from .typing import ValueType

class SnmpError(Exception): ...
class SnmpEncodeError(SnmpError): ...
class SnmpDecodeError(SnmpError): ...
class NoSuchInstance(SnmpError): ...

class GetNextIter(object):
    def __init__(self: "GetNextIter", oid: str) -> None: ...

class GetBulkIter(object):
    def __init__(
        self: "GetBulkIter", oid: str, max_repetitions: int
    ) -> None: ...

class SnmpV1ClientSocket(object):
    def __init__(
        self: "SnmpV1ClientSocket",
        addr: str,
        community: str,
        version: int,
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
    ) -> None: ...
    def get_fd(self: "SnmpV1ClientSocket") -> int: ...
    def send_get(self: "SnmpV1ClientSocket", oid: str) -> None: ...
    def send_get_many(self: "SnmpV1ClientSocket", oids: List[str]) -> None: ...
    def send_getnext(
        self: "SnmpV1ClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def send_getbulk(
        self: "SnmpV1ClientSocket", iter_getbulk: GetBulkIter
    ) -> None: ...
    def recv_getresponse(self: "SnmpV1ClientSocket") -> ValueType: ...
    def recv_getresponse_many(
        self: "SnmpV1ClientSocket",
    ) -> Dict[str, ValueType]: ...
    def recv_getresponse_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    def recv_getresponse_bulk(
        self: "SnmpV1ClientSocket", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...

class SnmpV2cClientSocket(object):
    def __init__(
        self: "SnmpV2cClientSocket",
        addr: str,
        community: str,
        version: int,
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
    ) -> None: ...
    def get_fd(self: "SnmpV2cClientSocket") -> int: ...
    def send_get(self: "SnmpV2cClientSocket", oid: str) -> None: ...
    def send_get_many(
        self: "SnmpV2cClientSocket", oids: List[str]
    ) -> None: ...
    def send_getnext(
        self: "SnmpV2cClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def send_getbulk(
        self: "SnmpV2cClientSocket", iter_getbulk: GetBulkIter
    ) -> None: ...
    def recv_getresponse(self: "SnmpV2cClientSocket") -> ValueType: ...
    def recv_getresponse_many(
        self: "SnmpV2cClientSocket",
    ) -> Dict[str, ValueType]: ...
    def recv_getresponse_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    def recv_getresponse_bulk(
        self: "SnmpV2cClientSocket", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...
