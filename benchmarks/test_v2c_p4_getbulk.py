# ---------------------------------------------------------------------
# Gufo SNMP: Getbulk benchmarks (4-thread parallel)
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio
import concurrent.futures

# Third-party modules
# from pysnmp.hlapi import v3arch
from easysnmp import snmp_bulkwalk as e_snmp_bulkwalk

# Gufo SNMP modules
from gufo.snmp.async_client import SnmpSession as AsyncSnmpSession
from gufo.snmp.snmpd import Snmpd
from gufo.snmp.sync_client import SnmpSession as SyncSnmpSession

BASE_OID = "1.3.6"
SNMP_COMMUNITY = "public"
CONCURRENCY = 4


def run_on_threadpool(fn):
    with concurrent.futures.ThreadPoolExecutor(
        max_workers=CONCURRENCY
    ) as executor:
        futures = [executor.submit(fn) for _ in range(CONCURRENCY)]
        concurrent.futures.wait(futures)


async def run_async(fn):
    tasks = [asyncio.create_task(fn()) for _ in range(CONCURRENCY)]
    await asyncio.gather(*tasks)


def test_gufo_snmp_sync(snmpd: Snmpd, benchmark) -> None:
    def do_test():
        with SyncSnmpSession(
            addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
        ) as session:
            for _k, _v in session.getbulk(BASE_OID):
                pass

    @benchmark
    def bench():
        run_on_threadpool(do_test)


def test_gufo_snmp_async(snmpd: Snmpd, benchmark) -> None:
    @benchmark
    def bench():
        async def inner():
            async with AsyncSnmpSession(
                addr=snmpd.address, port=snmpd.port, community=SNMP_COMMUNITY
            ) as session:
                async for _k, _v in session.getbulk(BASE_OID):
                    pass

        asyncio.run(run_async(inner))


def test_easysnmp_sync(snmpd: Snmpd, benchmark) -> None:
    def do_test():
        for item in e_snmp_bulkwalk(
            oids=BASE_OID,
            community=SNMP_COMMUNITY,
            hostname=snmpd.address,
            version=2,
            remote_port=snmpd.port,
            use_numeric=True,
        ):
            _ = item.oid  # Force deserialization
            _ = item.value  # Force deserialization

    @benchmark
    def bench():
        run_on_threadpool(do_test)
