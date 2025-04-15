# ---------------------------------------------------------------------
# Gufo SNMP: Collect PGO data
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""Run variuous workload to collect Profile Guided Optimization data."""

# Python modules
import socket

# Gufo SNMP Modules
from gufo.snmp.user import User, Md5Key, KeyType, DesKey, Aes128Key, Sha1Key
from gufo.snmp.snmpd import Snmpd
from gufo.snmp.sync_client import SnmpSession as SyncSnmpSession

SNMPD_ADDRESS = "127.0.0.1"
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

BASE_OID = "1.3.6"
OID1 = "1.3.6.1.2.1.1.6.0"
OID2 = "1.3.6.1.2.1.1.5.0"


def get_free_port() -> int:
    """Get free UDP port."""
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.bind(("127.0.0.1", 0))  # Bind to a random available port
        return sock.getsockname()[1]  # Get assigned port


def run_get_v2c_sync(snmpd: Snmpd) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
    ) as session:
        for _ in range(100):
            session.get(OID1)
            session.get(OID2)


def run_get_v3_sync(snmpd: Snmpd, user: User) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, user=user
    ) as session:
        for _ in range(100):
            session.get(OID1)
            session.get(OID2)


def run_getmany_v2c_sync(snmpd: Snmpd) -> None:
    oids = [OID1, OID2]
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
    ) as session:
        for _ in range(100):
            session.get_many(oids)


def run_getmany_v3_sync(snmpd: Snmpd, user: User) -> None:
    oids = [OID1, OID2]
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, user=user
    ) as session:
        for _ in range(100):
            session.get_many(oids)


def run_getnext_v2c_sync(snmpd: Snmpd) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
    ) as session:
        for _ in session.getnext(BASE_OID):
            pass


def run_getnext_v3_sync(snmpd: Snmpd, user: User) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, user=user
    ) as session:
        for _ in session.getnext(BASE_OID):
            pass


def run_getbulk_v2c_sync(snmpd: Snmpd) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
    ) as session:
        for _ in session.getbulk(BASE_OID):
            pass


def run_getbulk_v3_sync(snmpd: Snmpd, user: User) -> None:
    with SyncSnmpSession(
        addr=snmpd.address, port=snmpd.port, user=user
    ) as session:
        for _ in session.getbulk(BASE_OID):
            pass


def main() -> None:
    with Snmpd(
        path=SNMPD_PATH,
        address=SNMPD_ADDRESS,
        port=get_free_port(),
        community=SNMP_COMMUNITY,
        location=SNMP_LOCATION,
        contact=SNMP_CONTACT,
        users=SNMP_USERS,
    ) as snmpd:
        run_get_v2c_sync(snmpd)
        run_getmany_v2c_sync(snmpd)
        run_getnext_v2c_sync(snmpd)
        run_getbulk_v2c_sync(snmpd)
        for user in SNMP_USERS:
            run_get_v3_sync(snmpd, user)
            run_getmany_v3_sync(snmpd, user)
            run_getnext_v3_sync(snmpd, user)
            run_getbulk_v3_sync(snmpd, user)


if __name__ == "__main__":
    main()
