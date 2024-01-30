# ---------------------------------------------------------------------
# Gufo Labs: Test Gufo SNMP async client
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio
from typing import Any, Dict, Optional, cast

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.snmp import NoSuchInstance, ValueType
from gufo.snmp.async_client import SnmpSession
from gufo.snmp.snmpd import Snmpd

from .util import (
    ALL,
    SNMP_CONTACT,
    SNMP_LOCATION,
    SNMPD_ADDRESS,
    SNMPD_PORT,
    V1,
    V2,
    V3,
    ids,
)


def test_snmpd_version(snmpd: Snmpd) -> None:
    assert snmpd.version


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_timeout_get(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    async def inner() -> ValueType:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT + 1,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            return await session.get("1.3.6.1.2.1.1")

    with pytest.raises(TimeoutError):
        asyncio.run(inner())


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_timeout_get_many(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    async def inner() -> Dict[str, ValueType]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT + 1,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            return await session.get_many(["1.3.6.1.2.1.1"])

    with pytest.raises(TimeoutError):
        asyncio.run(inner())


async def snmp_get(
    cfg: Dict[str, Any], engine_id: Optional[bytes], oid: str
) -> ValueType:
    async with SnmpSession(
        addr=SNMPD_ADDRESS,
        port=SNMPD_PORT,
        timeout=1.0,
        engine_id=engine_id,
        **cfg,
    ) as session:
        return await session.get(oid)


# Uncomment for single config check
# def test_xxx(snmpd: Snmpd):
#     asyncio.run(snmp_get(V3[0], None, "1.3.6.1.2.1.1.6.0"))


@pytest.mark.parametrize("cfg", ALL, ids=ids)
@pytest.mark.parametrize(
    ("oid", "expected"),
    [
        ("1.3.6.1.2.1.1.6.0", SNMP_LOCATION.encode()),
        ("1.3.6.1.2.1.1.4.0", SNMP_CONTACT.encode()),
    ],
)
def test_get(
    cfg: Dict[str, Any], oid: str, expected: ValueType, snmpd: Snmpd
) -> None:
    r = asyncio.run(snmp_get(cfg, snmpd.engine_id, oid))
    assert r == expected


@pytest.mark.parametrize("cfg", V3, ids=ids)
@pytest.mark.parametrize(
    ("oid", "expected"),
    [
        ("1.3.6.1.2.1.1.6.0", SNMP_LOCATION.encode()),
        ("1.3.6.1.2.1.1.4.0", SNMP_CONTACT.encode()),
    ],
)
def test_get_wo_engine_id(
    cfg: Dict[str, Any], oid: str, expected: ValueType, snmpd: Snmpd
) -> None:
    r = asyncio.run(snmp_get(cfg, None, oid))
    assert r == expected


@pytest.mark.parametrize("cfg", V2 + V3, ids=ids)
def test_get_nosuchinstance(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    with pytest.raises(NoSuchInstance):
        asyncio.run(snmp_get(cfg, snmpd.engine_id, "1.3.6.1.2.1.1.6"))


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_sys_uptime(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """sysUptime.0 returns TimeTicks type."""
    r = asyncio.run(snmp_get(cfg, snmpd.engine_id, "1.3.6.1.2.1.1.3.0"))
    assert isinstance(r, int)


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_sys_objectid(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """sysObjectId.0 returns OBJECT IDENTIFIER type."""
    r = asyncio.run(snmp_get(cfg, snmpd.engine_id, "1.3.6.1.2.1.1.2.0"))
    assert isinstance(r, str)
    assert r.startswith("1.3.6.1.4.1.8072.3.2.")


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_get_many(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    async def inner() -> Dict[str, ValueType]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
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
    assert cast(str, r["1.3.6.1.2.1.1.2.0"]).startswith(
        "1.3.6.1.4.1.8072.3.2."
    )
    assert "1.3.6.1.2.1.1.3.0" in r
    assert isinstance(r["1.3.6.1.2.1.1.3.0"], int)
    assert "1.3.6.1.2.1.1.6.0" in r
    assert r["1.3.6.1.2.1.1.6.0"] == SNMP_LOCATION.encode()
    assert "1.3.6.1.2.1.1.4.0" in r
    assert r["1.3.6.1.2.1.1.4.0"] == SNMP_CONTACT.encode()


@pytest.mark.parametrize("cfg", V2 + V3, ids=ids)
def test_get_many_skip(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    async def inner() -> Dict[str, Any]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
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


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_get_many_long_request(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    async def inner() -> Dict[str, Any]:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            return await session.get_many(oids)

    oids = [
        "1.3.6.1.2.1.1.1.0",
        "1.3.6.1.2.1.1.2.0",
        "1.3.6.1.2.1.1.3.0",
        "1.3.6.1.2.1.1.4.0",
        "1.3.6.1.2.1.1.5.0",
        "1.3.6.1.2.1.1.6.0",
        "1.3.6.1.2.1.1.7.0",
    ]
    r = asyncio.run(inner())
    assert len(r) == len(oids)
    for oid in oids:
        assert oid in r


@pytest.mark.parametrize("cfg", V1 + V2 + V3[:1], ids=ids)
def test_getnext(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """Iterate over whole MIB."""

    async def inner() -> int:
        n = 0
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            async for _ in session.getnext("1.3.6"):
                n += 1
        return n

    n = asyncio.run(inner())
    assert n > 0


@pytest.mark.parametrize("cfg", ALL, ids=ids)
def test_getnext_single(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """Test single value is returned with bulk."""

    async def inner() -> int:
        n = 0
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            async for oid, value in session.getnext("1.3.6.1.2.1.1.2"):
                assert oid == "1.3.6.1.2.1.1.2.0"
                assert value.startswith("1.3.6.1.4.1.8072.3.2.")
                n += 1
        return n

    n = asyncio.run(inner())
    assert n == 1


@pytest.mark.parametrize("cfg", V2 + V3, ids=ids)
def test_getbulk(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """Iterate over whole MIB."""

    async def inner() -> int:
        n = 0
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            async for _ in session.getbulk("1.3.6"):
                n += 1
        return n

    n = asyncio.run(inner())
    assert n > 0


@pytest.mark.parametrize("cfg", V2 + V3, ids=ids)
def test_getbulk_single(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
    """Test single value is returned with bulk."""

    async def inner() -> int:
        n = 0
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            async for oid, value in session.getbulk("1.3.6.1.2.1.1.2"):
                assert oid == "1.3.6.1.2.1.1.2.0"
                assert value.startswith("1.3.6.1.4.1.8072.3.2.")
                n += 1
        return n

    n = asyncio.run(inner())
    assert n == 1


# @pytest.mark.parametrize("cfg", V2)
# def test_getnext_getbulk(cfg: Dict[str, Any], snmpd: Snmpd) -> None:
#     """Cross-test of getnext and getbulk."""

#     def is_valid(oid: str) -> bool:
#         return not oid.startswith(("1.3.6.1.2.1.7.5.", "1.3.6.1.2.1.6.13"))

#     async def inner_getnext() -> Set[str]:
#         r: Set[str] = set()
#         async with SnmpSession(
#             addr=SNMPD_ADDRESS,
#             port=SNMPD_PORT,
#             timeout=1.0,
#             engine_id=snmpd.engine_id,
#             **cfg,
#         ) as session:
#             async for oid, _ in session.getnext("1.3.6"):
#                 if is_valid(oid):
#                     r.add(oid)
#         return r

#     async def inner_getbulk() -> Set[str]:
#         r: Set[str] = set()
#         async with SnmpSession(
#             addr=SNMPD_ADDRESS,
#             port=SNMPD_PORT,
#             timeout=1.0,
#             engine_id=snmpd.engine_id,
#             **cfg,
#         ) as session:
#             async for oid, _ in session.getbulk("1.3.6"):
#                 if is_valid(oid):
#                     r.add(oid)
#         return r

#     gn = asyncio.run(inner_getnext())
#     gb = asyncio.run(inner_getbulk())
#     diff = gn.symmetric_difference(gb)
#     assert diff == set()


@pytest.mark.parametrize("cfg", ALL, ids=ids)
@pytest.mark.parametrize("allow_bulk", [False, True])
def test_fetch(cfg: Dict[str, Any], allow_bulk: bool, snmpd: Snmpd) -> None:
    async def inner() -> None:
        async with SnmpSession(
            addr=SNMPD_ADDRESS,
            port=SNMPD_PORT,
            timeout=1.0,
            allow_bulk=allow_bulk,
            engine_id=snmpd.engine_id,
            **cfg,
        ) as session:
            n = 0
            async for _, _ in session.fetch("1.3.6.1.2.1.1"):
                n += 1
            assert n > 0

    asyncio.run(inner())


@pytest.mark.parametrize("cfg", ALL, ids=ids)
@pytest.mark.parametrize("allow_bulk", [False, True])
def test_fetch_file_not_found(
    cfg: Dict[str, Any], allow_bulk: bool, snmpd: Snmpd
) -> None:
    """Test issue #1 condition."""

    async def inner() -> None:
        for _ in range(10):
            async with SnmpSession(
                addr=SNMPD_ADDRESS,
                port=SNMPD_PORT,
                allow_bulk=allow_bulk,
                engine_id=snmpd.engine_id,
                **cfg,
            ) as session:
                async for _ in session.fetch("1.3.6.1.2.1.1.3"):
                    pass

    asyncio.run(inner())


@pytest.mark.parametrize("cfg", V3, ids=ids)
def test_get_engine_id(snmpd: Snmpd, cfg: Dict[str, Any]) -> None:
    async def inner() -> bytes:
        async with SnmpSession(
            addr=SNMPD_ADDRESS, port=SNMPD_PORT, timeout=1.0, **cfg
        ) as session:
            return session.get_engine_id()

    r = asyncio.run(inner())
    assert r == snmpd.engine_id
