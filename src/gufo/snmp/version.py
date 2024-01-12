# ---------------------------------------------------------------------
# Gufo SNMP: SnmpVersion definitionn
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""SnmpVersion definition."""

# Python modules
import enum


class SnmpVersion(enum.IntEnum):
    """SNMP protocol version."""

    v1 = 0
    v2c = 1
    v3 = 3
