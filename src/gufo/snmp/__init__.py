# ---------------------------------------------------------------------
# Gufo SNMP: Python SNMP Library
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Gufo SNMP: The accelerated Python asyncio SNMP client library.

Attributes:
    __version__: Current version
"""

# Gufo Labs modules
from ._fast import NoSuchInstance, SnmpDecodeError, SnmpEncodeError, SnmpError
from .client import SnmpSession
from .typing import ValueType
from .version import SnmpVersion

__version__: str = "0.1.0"
__all__ = [
    "__version__",
    "SnmpSession",
    "SnmpVersion",
    "SnmpError",
    "SnmpEncodeError",
    "SnmpDecodeError",
    "NoSuchInstance",
    "ValueType",
]
