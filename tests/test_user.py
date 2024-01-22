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
from gufo.snmp.user import BaseAuthKey, KeyType, Md5Key, Sha1Key

AUTH = [Md5Key, Sha1Key]


@pytest.mark.parametrize("kls", AUTH)
def test_auth_subclass(kls: Type[object]) -> None:
    assert issubclass(kls, BaseAuthKey), "Must be subclass of BaseKey"


@pytest.mark.parametrize(
    "key_type", [KeyType.Password, KeyType.Master, KeyType.Localized]
)
@pytest.mark.parametrize("kls", AUTH)
@pytest.mark.parametrize(
    "key",
    [
        b"",
        b"\x01\x02",
        b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10",
        b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12",
        b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15",
    ],
)
def test_key_padding(
    kls: Type[BaseAuthKey], key: bytes, key_type: KeyType
) -> None:
    k = kls(key, key_type=key_type)
    if key_type == KeyType.Password:
        assert len(k.key) == len(key)
    else:
        assert len(k.key) == k.KEY_LENGTH


@pytest.mark.parametrize(
    ("auth", "passwd", "expected"),
    [
        (
            Md5Key,
            b"maplesyrup",
            b"\x9f\xaf\x32\x83\x88\x4e\x92\x83\x4e\xbc\x98\x47\xd8\xed\xd9\x63",
        ),
        (
            Sha1Key,
            b"maplesyrup",
            b"\x9f\xb5\xcc\x03\x81\x49\x7b\x37\x93\x52\x89\x39\xff\x78\x8d\x5d\x79\x14\x52\x11",
        ),
    ],
)
def test_get_master_key(
    auth: BaseAuthKey, passwd: bytes, expected: bytes
) -> None:
    r = auth.get_master_key(passwd)
    assert r == expected


@pytest.mark.parametrize(
    ("auth", "master_key", "engine_id", "expected"),
    [
        (
            Md5Key,
            b"\x9f\xaf\x32\x83\x88\x4e\x92\x83\x4e\xbc\x98\x47\xd8\xed\xd9\x63",
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02",
            b"\x52\x6f\x5e\xed\x9f\xcc\xe2\x6f\x89\x64\xc2\x93\x07\x87\xd8\x2b",
        ),
        (
            Sha1Key,
            b"\x9f\xb5\xcc\x03\x81\x49\x7b\x37\x93\x52\x89\x39\xff\x78\x8d\x5d\x79\x14\x52\x11",
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02",
            b"\x66\x95\xfe\xbc\x92\x88\xe3\x62\x82\x23\x5f\xc7\x15\x1f\x12\x84\x97\xb3\x8f\x3f",
        ),
    ],
)
def test_get_localized_key(
    auth: BaseAuthKey, master_key: bytes, engine_id: bytes, expected: bytes
) -> None:
    r = auth.get_localized_key(master_key, engine_id)
    assert r == expected
