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
class SnmpAuthError(SnmpError): ...  # v3 only
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
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
        timeout_ns: int,
    ) -> None: ...
    def get_fd(self: "SnmpV1ClientSocket") -> int: ...

    # .get()
    def get(self: "SnmpV1ClientSocket", oid: str) -> ValueType: ...
    def send_get(self: "SnmpV1ClientSocket", oid: str) -> None: ...
    def recv_get(self: "SnmpV1ClientSocket") -> ValueType: ...

    # .get_many()
    def get_many(
        self: "SnmpV1ClientSocket", oids: List[str]
    ) -> Dict[str, ValueType]: ...
    def send_get_many(self: "SnmpV1ClientSocket", oids: List[str]) -> None: ...
    def recv_get_many(
        self: "SnmpV1ClientSocket",
    ) -> Dict[str, ValueType]: ...

    # .get_next
    def get_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...

    # .get_bulk()
    def get_bulk(
        self: "SnmpV1ClientSocket", iter_getbulk: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...
    def send_get_bulk(
        self: "SnmpV1ClientSocket", iter_getbulk: GetBulkIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV1ClientSocket", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...

class SnmpV2cClientSocket(object):
    def __init__(
        self: "SnmpV2cClientSocket",
        addr: str,
        community: str,
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
        timeout_ns: int,
    ) -> None: ...
    def get_fd(self: "SnmpV2cClientSocket") -> int: ...
    # .get()
    def get(self: "SnmpV2cClientSocket", oid: str) -> ValueType: ...
    def send_get(self: "SnmpV2cClientSocket", oid: str) -> None: ...
    def recv_get(self: "SnmpV2cClientSocket") -> ValueType: ...
    # .get_many
    def get_many(
        self: "SnmpV2cClientSocket", oids: List[str]
    ) -> Dict[str, ValueType]: ...
    def send_get_many(
        self: "SnmpV2cClientSocket", oids: List[str]
    ) -> None: ...
    def recv_get_many(
        self: "SnmpV2cClientSocket",
    ) -> Dict[str, ValueType]: ...
    # .get_next()
    def get_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    # .get_bulk()
    def get_bulk(
        self: "SnmpV2cClientSocket", iter_getbulk: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...
    def send_get_bulk(
        self: "SnmpV2cClientSocket", iter_getbulk: GetBulkIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV2cClientSocket", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...

class SnmpV3ClientSocket(object):
    def __init__(
        self: "SnmpV3ClientSocket",
        addr: str,
        engine_id: bytes,
        user_name: str,
        auth_alg: int,
        auth_key: bytes,
        priv_alg: int,
        priv_key: bytes,
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
        timeout_ns: int,
    ) -> None: ...
    def set_keys(
        self: "SnmpV3ClientSocket",
        user_name: str,
        auth_alg: int,
        auth_key: bytes,
        priv_alg: int,
        priv_key: bytes,
    ) -> None: ...
    def get_fd(self: "SnmpV3ClientSocket") -> int: ...
    def get_engine_id(self: "SnmpV3ClientSocket") -> bytes: ...
    # .get()
    def get(self: "SnmpV3ClientSocket", oid: str) -> ValueType: ...
    def send_get(self: "SnmpV3ClientSocket", oid: str) -> None: ...
    def recv_get(self: "SnmpV3ClientSocket") -> ValueType: ...
    # .get_many()
    def send_get_many(self: "SnmpV3ClientSocket", oids: List[str]) -> None: ...
    def recv_get_many(
        self: "SnmpV3ClientSocket",
    ) -> Dict[str, ValueType]: ...
    def get_many(
        self: "SnmpV3ClientSocket", oids: List[str]
    ) -> Dict[str, ValueType]: ...
    # .get_next
    def get_next(
        self: "SnmpV3ClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV3ClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV3ClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
    # Rest
    def get_bulk(
        self: "SnmpV3ClientSocket", iter_getbulk: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...
    def send_get_bulk(
        self: "SnmpV3ClientSocket", iter_getbulk: GetBulkIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV3ClientSocket", iter_getnext: GetBulkIter
    ) -> List[Tuple[str, ValueType]]: ...
    # .refresh
    def async_recv_refresh(self: "SnmpV3ClientSocket") -> None: ...
    def async_send_refresh(self: "SnmpV3ClientSocket") -> None: ...
    def sync_refresh(self: "SnmpV3ClientSocket") -> None: ...

def get_master_key(auth_alg: int, passwd: bytes) -> bytes: ...
def get_localized_key(
    auth_alg: int, passwd: bytes, engine_id: bytes
) -> bytes: ...
