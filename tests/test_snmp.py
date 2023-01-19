# ---------------------------------------------------------------------
# Gufo Labs: Test Gufo SNMP
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Any, Dict
import asyncio

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.snmp import SnmpSession, NoSuchInstance
from gufo.snmp.snmpd import Snmpd

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = 10161
SNMPD_PATH = "/usr/sbin/snmpd"
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_USER = "rouser"


@pytest.fixture(scope="module")
def snmpd():
    with Snmpd(
        path=SNMPD_PATH,
        address=SNMPD_ADDRESS,
        port=SNMPD_PORT,
        community=SNMP_COMMUNITY,
        location=SNMP_LOCATION,
        contact=SNMP_CONTACT,
        user=SNMP_USER,
    ):
        yield None


def test_timeout(snmpd):
    async def inner():
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT + 1,
            community=SNMP_COMMUNITY,
            timeout=1.0,
        ) as session:
            return await session.get("1.3.6.1.2.1.1")

    with pytest.raises(TimeoutError):
        asyncio.run(inner())


async def snmp_get(oid: str) -> Any:
    async with SnmpSession(
        addr=SNMPD_ADDRESS,
        port=SNMPD_PORT,
        community=SNMP_COMMUNITY,
        timeout=1.0,
    ) as session:
        return await session.get(oid)


@pytest.mark.parametrize(
    "oid,expected",
    [
        ("1.3.6.1.2.1.1.6.0", SNMP_LOCATION.encode()),
        ("1.3.6.1.2.1.1.4.0", SNMP_CONTACT.encode()),
    ],
)
def test_get(oid, expected, snmpd):
    r = asyncio.run(snmp_get(oid))
    assert r == expected


def test_get_nosuchinstance(snmpd):
    with pytest.raises(NoSuchInstance):
        asyncio.run(snmp_get("1.3.6.1.2.1.1.6"))


def test_sys_uptime(snmpd):
    """
    sysUptime.0 returns TimeTicks type
    """
    r = asyncio.run(snmp_get("1.3.6.1.2.1.1.3.0"))
    assert isinstance(r, int)


def test_sys_objectid(snmpd):
    """
    sysObjectId.0 returns OBJECT IDENTIFIER type
    """
    r = asyncio.run(snmp_get("1.3.6.1.2.1.1.2.0"))
    assert isinstance(r, str)
    assert r.startswith("1.3.6.1.4.1.8072.3.2.")


def test_get_many(snmpd):
    async def inner() -> Dict[str, Any]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            community=SNMP_COMMUNITY,
            timeout=1.0,
        ) as session:
            return await session.get_many(
                [
                    "1.3.6.1.2.1.1.2.0",
                    "1.3.6.1.2.1.1.3.0",
                    "1.3.6.1.2.1.1.6.0",
                    "1.3.6.1.2.1.1.4.0",
                ]
            )

    r = asyncio.run(inner())
    assert isinstance(r, dict)
    assert "1.3.6.1.2.1.1.2.0" in r
    assert r["1.3.6.1.2.1.1.2.0"].startswith("1.3.6.1.4.1.8072.3.2.")
    assert "1.3.6.1.2.1.1.3.0" in r
    assert isinstance(r["1.3.6.1.2.1.1.3.0"], int)
    assert "1.3.6.1.2.1.1.6.0" in r
    assert r["1.3.6.1.2.1.1.6.0"] == SNMP_LOCATION.encode()
    assert "1.3.6.1.2.1.1.4.0" in r
    assert r["1.3.6.1.2.1.1.4.0"] == SNMP_CONTACT.encode()


def test_get_many_skip(snmpd):
    async def inner() -> Dict[str, Any]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            community=SNMP_COMMUNITY,
            timeout=1.0,
        ) as session:
            return await session.get_many(
                [
                    "1.3.6.1.2.1.1.6",  # Missed
                    "1.3.6.1.2.1.1.2.0",
                    "1.3.6.1.2.1.1.3.0",
                    "1.3.6.1.2.1.1.6.0",
                    "1.3.6.1.2.1.1.4.0",
                ]
            )

    r = asyncio.run(inner())
    assert len(r) == 4
    assert "1.3.6.1.2.1.1.6" not in r
    assert "1.3.6.1.2.1.1.2.0" in r
    assert "1.3.6.1.2.1.1.3.0" in r
    assert "1.3.6.1.2.1.1.6.0" in r
    assert "1.3.6.1.2.1.1.4.0" in r


def test_getnext(snmpd):
    """
    Iterate over whole MIB
    """

    async def inner():
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            community=SNMP_COMMUNITY,
            timeout=1.0,
        ) as session:
            async for _ in session.getnext("1.3.6"):
                pass

    asyncio.run(inner())
