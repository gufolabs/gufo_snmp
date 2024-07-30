# ---------------------------------------------------------------------
# Gufo SNMP: Python SNMP Library
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
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

__version__: str = "0.5.2"
__all__ = [
    "__version__",
    "Aes128Key",
    "DesKey",
    "Md5Key",
    "Sha1Key",
    "SnmpAuthError",
    "SnmpSession",
    "SnmpVersion",
    "SnmpError",
    "SnmpEncodeError",
    "SnmpDecodeError",
    "NoSuchInstance",
    "User",
    "ValueType",
]
