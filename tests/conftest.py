# ---------------------------------------------------------------------
# Gufo SNMP: snmpd fixture
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import logging
from typing import Iterator

# Third-party modules
import pytest

# Gufo SNMP modules
from gufo.snmp.snmpd import Snmpd

from .util import (
    SNMP_COMMUNITY,
    SNMP_CONTACT,
    SNMP_LOCATION,
    SNMP_USERS,
    SNMPD_ADDRESS,
    SNMPD_PATH,
    SNMPD_PORT,
)


@pytest.fixture(scope="session")
def snmpd() -> Iterator[Snmpd]:
    logger = logging.getLogger("gufo.snmp.snmpd")
    logger.setLevel(logging.DEBUG)
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
