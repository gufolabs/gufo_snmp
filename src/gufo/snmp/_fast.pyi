# ---------------------------------------------------------------------
# Gufo SNMP: _fast typing
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Dict, List, Optional, Tuple, Union

# Gufo Labs modules
from .typing import ValueType

class SnmpError(Exception): ...
class SnmpEncodeError(SnmpError): ...
class SnmpDecodeError(SnmpError): ...
class SnmpAuthError(SnmpError): ...  # v3 only
class NoSuchInstance(SnmpError): ...

class GetIter(object):
    def __init__(
        self: "GetIter", oid: str, max_repetitions: Optional[int] = None
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
        self: "SnmpV1ClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV1ClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...

    # .get_bulk()
    def get_bulk(
        self: "SnmpV1ClientSocket", iter_getbulk: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...
    def send_get_bulk(
        self: "SnmpV1ClientSocket", iter_getbulk: GetIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV1ClientSocket", iter_getnext: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...

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
        self: "SnmpV2cClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV2cClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...
    # .get_bulk()
    def get_bulk(
        self: "SnmpV2cClientSocket", iter_getbulk: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...
    def send_get_bulk(
        self: "SnmpV2cClientSocket", iter_getbulk: GetIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV2cClientSocket", iter_getnext: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...

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
        self: "SnmpV3ClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...
    def send_get_next(
        self: "SnmpV3ClientSocket", iter_getnext: GetIter
    ) -> None: ...
    def recv_get_next(
        self: "SnmpV3ClientSocket", iter_getnext: GetIter
    ) -> Tuple[str, ValueType]: ...
    # Rest
    def get_bulk(
        self: "SnmpV3ClientSocket", iter_getbulk: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...
    def send_get_bulk(
        self: "SnmpV3ClientSocket", iter_getbulk: GetIter
    ) -> None: ...
    def recv_get_bulk(
        self: "SnmpV3ClientSocket", iter_getnext: GetIter
    ) -> List[Union[Tuple[str, ValueType], None]]: ...
    # .refresh
    def refresh(self: "SnmpV3ClientSocket") -> None: ...
    def send_refresh(self: "SnmpV3ClientSocket") -> None: ...
    def recv_refresh(self: "SnmpV3ClientSocket") -> None: ...

def get_master_key(auth_alg: int, passwd: bytes) -> bytes: ...
def get_localized_key(
    auth_alg: int, passwd: bytes, engine_id: bytes
) -> bytes: ...
