# ---------------------------------------------------------------------
# Gufo SNMP: Authentication primitives test
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Type

# Third-party modules
import pytest

# Gufo SNMP modules
from gufo.snmp.user import BaseAuthKey, Md5Key, Sha1Key

AUTH = [Md5Key, Sha1Key]


@pytest.mark.parametrize("kls", AUTH)
def test_auth_subclass(kls: Type[object]) -> None:
    assert issubclass(kls, BaseAuthKey), "Must be subclass of BaseKey"


@pytest.mark.parametrize("kls", AUTH)
@pytest.mark.parametrize(
    ("key", "expected"),
    [
        (
            b"",
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        ),
        (
            b"\x01\x02",
            b"\x01\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        ),
        (
            b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10",
            b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10",
        ),
        (
            b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12",
            b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10",
        ),
    ],
)
def test_auth_key(kls: Type[BaseAuthKey], key: bytes, expected: bytes) -> None:
    k = kls(key)
    assert k.key == expected
