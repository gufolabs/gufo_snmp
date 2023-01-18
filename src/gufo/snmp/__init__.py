# ---------------------------------------------------------------------
# Gufo SNMP: Python SNMP Library
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""
Attributes:
    __version__: Current version
"""

from ._fast import SNMPError, SNMPEncodeError, SNMPDecodeError, NoSuchInstance
from .client import SnmpSession

__version__: str = "0.1.0"
__all__ = [
    "__version__",
    "SnmpSession",
    "SNMPError",
    "SNMPEncodeError",
    "SNMPDecodeError",
    "NoSuchInstance",
]
