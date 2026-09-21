# ---------------------------------------------------------------------
# Gufo SNMP: Python SNMP Library
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
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
from .typing import ValueType
from .user import (
    Aes128Key,
    BaseAuthKey,
    BasePrivKey,
    DesKey,
    KeyExpansion,
    Md5Key,
    Sha1Key,
    Sha224Key,
    Sha256Key,
    Sha384Key,
    Sha512Key,
    User,
)
from .version import SnmpVersion

__version__: str = "0.12.0"
__all__ = [
    "Aes128Key",
    "BaseAuthKey",
    "BasePrivKey",
    "DesKey",
    "KeyExpansion",
    "Md5Key",
    "NoSuchInstance",
    "Sha1Key",
    "Sha224Key",
    "Sha256Key",
    "Sha384Key",
    "Sha512Key",
    "SnmpAuthError",
    "SnmpDecodeError",
    "SnmpEncodeError",
    "SnmpError",
    "SnmpVersion",
    "User",
    "ValueType",
    "__version__",
]
