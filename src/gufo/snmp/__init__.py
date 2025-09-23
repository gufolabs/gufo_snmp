# ---------------------------------------------------------------------
# Gufo SNMP: Python SNMP Library
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Gufo SNMP: The accelerated Python SNMP client library.

Attributes:
    __version__: Current version
"""

# Gufo Labs modules
from ._fast import (
    NoSuchInstance,
    SnmpAuthError,
    SnmpDecodeError,
    SnmpEncodeError,
    SnmpError,
)
from .async_client import SnmpSession
from .typing import ValueType
from .user import Aes128Key, DesKey, Md5Key, Sha1Key, User
from .version import SnmpVersion

__version__: str = "0.9.0"
__all__ = [
    "Aes128Key",
    "DesKey",
    "Md5Key",
    "NoSuchInstance",
    "Sha1Key",
    "SnmpAuthError",
    "SnmpDecodeError",
    "SnmpEncodeError",
    "SnmpError",
    "SnmpSession",
    "SnmpVersion",
    "User",
    "ValueType",
    "__version__",
]
