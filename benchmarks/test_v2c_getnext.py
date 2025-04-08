# ---------------------------------------------------------------------
# Gufo SNMP: Getnext benchmarks
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio

# Third-party modules
# from pysnmp.hlapi import v3arch
# Gufo SNMP modules
from gufo.snmp.async_client import SnmpSession as AsyncSnmpSession
from gufo.snmp.snmpd import Snmpd
from gufo.snmp.sync_client import SnmpSession as SyncSnmpSession

BASE_OID = "1.3.6"
SNMP_COMMUNITY = "public"


def test_gufo_snmp_sync(snmpd: Snmpd, benchmark) -> None:
    @benchmark
    def bench():
        with SyncSnmpSession(
            addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
        ) as session:
            for _k, _v in session.getnext(BASE_OID):
                pass


def test_gufo_snmp_async(snmpd: Snmpd, benchmark) -> None:
    @benchmark
    def bench():
        async def inner():
            async with AsyncSnmpSession(
                addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
            ) as session:
                async for _k, _v in session.getnext(BASE_OID):
                    pass

        asyncio.run(inner())
