# ---------------------------------------------------------------------
# Gufo SNMP: Benchmarks configuration
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import random
from typing import Iterator

# Third-party modules
import pytest

# Gufo SNMP modules
from gufo.snmp.snmpd import Snmpd
from gufo.snmp.user import Aes128Key, KeyType, Sha1Key, User

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = random.randint(52000, 53999)
SNMPD_PATH = "/usr/sbin/snmpd"
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_USERS = [
    User(
        name="user22",
        auth_key=Sha1Key(b"user22key", key_type=KeyType.Master),
        priv_key=Aes128Key(b"USER22KEY", key_type=KeyType.Master),
    ),
]


@pytest.fixture(scope="session")
def snmpd() -> Iterator[Snmpd]:
    with Snmpd(
        path=SNMPD_PATH,
        address=SNMPD_ADDRESS,
        port=SNMPD_PORT,
        community=SNMP_COMMUNITY,
        location=SNMP_LOCATION,
        contact=SNMP_CONTACT,
        users=SNMP_USERS,
        # Uncomment for debugging
        # verbose=True,
        # log_packets=True,
    ) as snmpd:
        yield snmpd
