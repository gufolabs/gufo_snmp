# ---------------------------------------------------------------------
# Gufo Labs: Test _fast module
# ---------------------------------------------------------------------
# Copyright (C) 2022-2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.snmp._fast import SnmpClientSocket


def test_invalid_version() -> None:
    with pytest.raises(ValueError, match="invalid version"):
        SnmpClientSocket("127.0.0.1:161", "public", 15, 0, 0, 0)


def test_invalid_address() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpClientSocket("127.0.0.500:161", "public", 1, 0, 0, 0)


def test_invalid_port() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpClientSocket("127.0.0.1:100000", "public", 1, 0, 0, 0)


def test_set_tos() -> None:
    SnmpClientSocket("127.0.0.1:161", "public", 1, 10, 0, 0)


def test_get_fd() -> None:
    SnmpClientSocket("127.0.0.1:161", "public", 1, 0, 0, 0).get_fd()
