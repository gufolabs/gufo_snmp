# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 GETBULK benchmarks
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio

# Third-party modules
# from pysnmp.hlapi import v3arch
from easysnmp import snmp_bulkwalk as e_snmp_bulkwalk

# Gufo SNMP modules
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


def test_gufo_snmp_sync(snmpd: Snmpd, benchmark) -> None:
    @benchmark
    def bench():
        with SyncSnmpSession(
            addr=snmpd.address,
            port=snmpd.port,
            version=SnmpVersion.v3,
            user=SNMP_USER,
        ) as session:
            for _k, _v in session.getbulk(BASE_OID):
                pass


def test_gufo_snmp_async(snmpd: Snmpd, benchmark) -> None:
    async def inner():
        async with AsyncSnmpSession(
            addr=snmpd.address,
            port=snmpd.port,
            version=SnmpVersion.v3,
            user=SNMP_USER,
        ) as session:
            async for _k, _v in session.getbulk(BASE_OID):
                pass

    @benchmark
    def bench():
        asyncio.run(inner())


# Fails with ValueError: embedded null character
# Need to found the way to tell the library that
# the password's type is a `MASTER`
def __test_easysnmp_sync(snmpd: Snmpd, benchmark) -> None:
    user_name = SNMP_USER.name
    privacy_key = SNMP_USER.priv_key.key.decode()
    auth_key = SNMP_USER.auth_key.key.decode()

    @benchmark
    def bench():
        for item in e_snmp_bulkwalk(
            oids=BASE_OID,
            security_level="authPriv",
            security_username=user_name,
            privacy_protocol="SHA1",
            privacy_password=privacy_key,
            auth_protocol="AES128",
            auth_password=auth_key,
            hostname=snmpd.address,
            version=3,
            remote_port=snmpd.port,
            use_numeric=True,
        ):
            _ = item.oid  # Force deserialization
            _ = item.value  # Force deserialization


def test_pysnmp_async(snmpd: Snmpd, benchmark) -> None:
    from pysnmp.hlapi.v3arch.asyncio import (
        USM_AUTH_HMAC96_SHA,
        USM_KEY_TYPE_MASTER,
        USM_PRIV_CFB128_AES,
        ContextData,
        ObjectIdentity,
        ObjectType,
        SnmpEngine,
        UdpTransportTarget,
        UsmUserData,
        bulk_walk_cmd,
    )

    user_name = SNMP_USER.name
    privacy_key = SNMP_USER.priv_key.key.decode()
    auth_key = SNMP_USER.auth_key.key.decode()

    async def inner() -> None:
        async for x, y, z, var_binds in bulk_walk_cmd(
            SnmpEngine(),
            UsmUserData(
                user_name,
                authProtocol=USM_AUTH_HMAC96_SHA,
                authKey=auth_key,
                authKeyType=USM_KEY_TYPE_MASTER,
                privProtocol=USM_PRIV_CFB128_AES,
                privKey=privacy_key,
                privKeyType=USM_KEY_TYPE_MASTER,
            ),
            await UdpTransportTarget.create((snmpd.address, snmpd.port)),
            ContextData(),
            0,
            25,
            ObjectType(ObjectIdentity(BASE_OID)),
        ):
            for i in var_binds:
                i.prettyPrint()

    @benchmark
    def bench():
        asyncio.run(inner())
