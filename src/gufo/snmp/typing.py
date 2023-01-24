# ---------------------------------------------------------------------
# Gufo SNMP: Types definitions
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Types definition.

Attributes:
    ValueType: Return type for SNMP query operations.
"""

# Python modules
from typing import Union

ValueType = Union[None, str, bytes, int, float]
