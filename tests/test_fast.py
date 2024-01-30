# ---------------------------------------------------------------------
# Gufo Labs: Test _fast module
# ---------------------------------------------------------------------
# Copyright (C) 2022-2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.snmp._fast import SnmpV1ClientSocket, SnmpV2cClientSocket


def test_v1_invalid_address() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpV1ClientSocket("127.0.0.500:161", "public", 0, 0, 0, 0)


def test_v2c_invalid_address() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpV2cClientSocket("127.0.0.500:161", "public", 0, 0, 0, 0)


def test_v1_invalid_port() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpV1ClientSocket("127.0.0.1:100000", "public", 0, 0, 0, 0)


def test_v2c_invalid_port() -> None:
    with pytest.raises(OSError, match="invalid address"):
        SnmpV2cClientSocket("127.0.0.1:100000", "public", 0, 0, 0, 0)


def test_v1_set_tos() -> None:
    SnmpV1ClientSocket("127.0.0.1:161", "public", 10, 0, 0, 0)


def test_v2c_set_tos() -> None:
    SnmpV2cClientSocket("127.0.0.1:161", "public", 10, 0, 0, 0)


def test_v1_get_fd() -> None:
    SnmpV1ClientSocket("127.0.0.1:161", "public", 0, 0, 0, 0).get_fd()


def test_v2c_get_fd() -> None:
    SnmpV2cClientSocket("127.0.0.1:161", "public", 0, 0, 0, 0).get_fd()
