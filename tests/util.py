# ---------------------------------------------------------------------
# Gufo SNMP: User definitions and test utilities.
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import random
from typing import Any

# Gufo SNMP Modules
from gufo.snmp import SnmpVersion
from gufo.snmp.user import Aes128Key, DesKey, KeyType, Md5Key, Sha1Key, User

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = random.randint(52000, 53999)
SNMPD_PATH = "/usr/sbin/snmpd"
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_USERS = [
    User(name="user00"),
    # MD5
    User(
        name="user10", auth_key=Md5Key(b"user10key", key_type=KeyType.Master)
    ),
    User(
        name="user10p",
        auth_key=Md5Key(b"user10pass", key_type=KeyType.Password),
    ),
    User(
        name="user11",
        auth_key=Md5Key(b"user11key", key_type=KeyType.Master),
        priv_key=DesKey(b"USER11KEY", key_type=KeyType.Master),
    ),
    User(
        name="user11p",
        auth_key=Md5Key(b"user11pass", key_type=KeyType.Password),
        priv_key=DesKey(b"USER11PASS", key_type=KeyType.Password),
    ),
    User(
        name="user12",
        auth_key=Md5Key(b"user11key", key_type=KeyType.Master),
        priv_key=Aes128Key(b"USER12KEY", key_type=KeyType.Master),
    ),
    # SHA1
    User(
        name="user20", auth_key=Sha1Key(b"user20key", key_type=KeyType.Master)
    ),
    User(
        name="user21",
        auth_key=Sha1Key(b"user21key", key_type=KeyType.Master),
        priv_key=DesKey(b"USER21KEY", key_type=KeyType.Master),
    ),
    User(
        name="user22",
        auth_key=Sha1Key(b"user22key", key_type=KeyType.Master),
        priv_key=Aes128Key(b"USER22KEY", key_type=KeyType.Master),
    ),
]

V1 = [{"version": SnmpVersion.v1, "community": SNMP_COMMUNITY}]
V2 = [{"version": SnmpVersion.v2c, "community": SNMP_COMMUNITY}]
V3 = [{"version": SnmpVersion.v3, "user": u} for u in SNMP_USERS]

ALL = V1 + V2 + V3


def ids(x: Any) -> str:
    if isinstance(x, dict) and "version" in x:
        r = [x["version"].name]
        user = x.get("user")
        if user:
            r += [user.name]
            if user.auth_key:
                r += [user.auth_key.__class__.__name__]
            if user.priv_key:
                r += [user.priv_key.__class__.__name__]
        return "-".join(r)
    return str(x)
