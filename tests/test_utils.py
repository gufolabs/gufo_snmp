# ---------------------------------------------------------------------
# Gufo SNMP: Utilities tests
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Third party modules
import pytest

# Gufo SNMP modules
from gufo.snmp.utils import format_sock_addr


@pytest.mark.parametrize(
    ("addr", "port", "expected"),
    [
        ("127.0.0.1", 161, "127.0.0.1:161"),
        ("192.0.2.1", 1161, "192.0.2.1:1161"),
        ("localhost", 161, "localhost:161"),
        ("2001:db8::1", 161, "[2001:db8::1]:161"),
        ("2001:db8::1", 1161, "[2001:db8::1]:1161"),
        ("[2001:db8::1]", 161, "[2001:db8::1]:161"),
    ],
)
def test_format_sock_addr(addr: str, port: int, expected: str) -> None:
    assert format_sock_addr(addr, port) == expected
