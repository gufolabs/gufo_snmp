# ---------------------------------------------------------------------
# Gufo SNMP: _fast typing
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Dict, List, Tuple

# Gufo Labs modules
from .typing import ValueType

class SNMPError(Exception): ...
class SNMPEncodeError(SNMPError): ...
class SNMPDecodeError(SNMPError): ...
class NoSuchInstance(SNMPError): ...

class GetNextIter(object):
    def __init__(self: "GetNextIter", oid: str) -> None: ...

class SnmpClientSocket(object):
    def __init__(
        self: "SnmpClientSocket",
        addr: str,
        community: str,
        version: int,
        tos: int,
        send_buffer_size: int,
        recv_buffer_size: int,
    ) -> None: ...
    def get_fd(self: "SnmpClientSocket") -> int: ...
    def send_get(self: "SnmpClientSocket", oid: str) -> None: ...
    def send_get_many(self: "SnmpClientSocket", oids: List[str]) -> None: ...
    def send_getnext(
        self: "SnmpClientSocket", iter_getnext: GetNextIter
    ) -> None: ...
    def recv_getresponse(self: "SnmpClientSocket") -> ValueType: ...
    def recv_getresponse_many(
        self: "SnmpClientSocket",
    ) -> Dict[str, ValueType]: ...
    def recv_getresponse_next(
        self: "SnmpClientSocket", iter_getnext: GetNextIter
    ) -> Tuple[str, ValueType]: ...
