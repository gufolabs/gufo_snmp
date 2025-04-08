# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 Getbulk benchmarks (4-thread parallel)
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio
import concurrent.futures

# Third-party modules
# from pysnmp.hlapi import v3arch
from gufo.snmp import SnmpVersion
from gufo.snmp.async_client import SnmpSession as AsyncSnmpSession
from gufo.snmp.snmpd import Snmpd
from gufo.snmp.sync_client import SnmpSession as SyncSnmpSession
from gufo.snmp.user import Aes128Key, KeyType, Sha1Key, User

BASE_OID = "1.3.6"
SNMP_USER = User(
    name="user22",
    auth_key=Sha1Key(b"user22key", key_type=KeyType.Master),
    priv_key=Aes128Key(b"USER22KEY", key_type=KeyType.Master),
)
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
            addr=snmpd.address,
            port=snmpd.port,
            version=SnmpVersion.v3,
            user=SNMP_USER,
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
                addr=snmpd.address,
                port=snmpd.port,
                version=SnmpVersion.v3,
                user=SNMP_USER,
            ) as session:
                async for _k, _v in session.getbulk(BASE_OID):
                    pass

        asyncio.run(run_async(inner))
