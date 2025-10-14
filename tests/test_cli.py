# ---------------------------------------------------------------------
# Gufo SNMP: Cli tests
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import List

# Third-party modules
import pytest

# Gufo SNMP modules
from gufo.snmp import SnmpVersion
from gufo.snmp.cli import Cli, ExitCode
from gufo.snmp.snmpd import Snmpd

from .util import SNMP_COMMUNITY, SNMP_LOCATION_OID, SNMPD_ADDRESS, SNMPD_PORT


def test_die() -> None:
    with pytest.raises(SystemExit):
        Cli.die("die!")


@pytest.mark.parametrize(
    "args",
    [
        # Invalid version
        ["--version", "??", "127.0.0.1", "1.3.6"],
        # Mutually exclusive versions
        ["-v1", "-v2c", "127.0.0.1", "1.3.6"],
        # No community for v1/v2c
        ["127.0.0.1", "1.3.6"],
        ["-v1", "127.0.0.1", "1.3.6"],
        ["-v2c", "127.0.0.1", "1.3.6"],
        # No user for v3
        ["-v3", "127.0.0.1", "1.3.6"],
        # Invalid command
        ["-X", "GETWHAT", "-c", "public", "127.0.0.1", "1.3.6"],
        # Invalid port
        ["-p", "AAA", "127.0.0.1", "1.3.6"],
        # Invalid OID
        ["-c", "public", "127.0.0.1", ".1.3.6"],
    ],
)
def test_invalid_options(args: List[str]) -> None:
    with pytest.raises(SystemExit):
        Cli().run(args)


@pytest.mark.parametrize(
    ("args", "expected_str", "expected_version"),
    [
        # Default v2c
        (["-c", "public", "127.0.0.1", "1.3.6"], "v2c", SnmpVersion.v2c),
        # Short options
        (["-v1", "-c", "public", "127.0.0.1", "1.3.6"], "v1", SnmpVersion.v1),
        (
            ["-v2c", "-c", "public", "127.0.0.1", "1.3.6"],
            "v2c",
            SnmpVersion.v2c,
        ),
        (["-v3", "-u", "user", "127.0.0.1", "1.3.6"], "v3", SnmpVersion.v3),
        # Long option
        (
            ["--version=v1", "-c", "public", "127.0.0.1", "1.3.6"],
            "v1",
            SnmpVersion.v1,
        ),
        (
            ["--version=v2c", "-c", "public", "127.0.0.1", "1.3.6"],
            "v2c",
            SnmpVersion.v2c,
        ),
        (
            ["--version=v3", "-u", "user", "127.0.0.1", "1.3.6"],
            "v3",
            SnmpVersion.v3,
        ),
    ],
)
def test_parse_version(
    args: List[str], expected_str: str, expected_version: SnmpVersion
) -> None:
    ns = Cli.parse_args(args)
    assert ns.version
    assert ns.version == expected_str


@pytest.mark.parametrize(
    ("args", "expected"),
    [
        (["-c", "public", "127.0.0.1", "1.3.6"], 161),
        (["-c", "public", "-p", "10000", "127.0.0.1", "1.3.6"], 10000),
        (["-c", "public", "--port=10000", "127.0.0.1", "1.3.6"], 10000),
    ],
)
def test_parse_port(args: List[str], expected: int) -> None:
    ns = Cli.parse_args(args)
    assert ns.port
    assert ns.port == expected


@pytest.mark.parametrize(
    ("oid", "expected"),
    [
        ("1.3.6", True),
        (".1.3.6", False),
        ("1.3.6.", False),
        (SNMP_LOCATION_OID, True),
    ],
)
def test_is_valid_oid(oid: str, expected: bool) -> None:
    r = Cli.is_valid_oid(oid)
    assert r is expected


@pytest.mark.parametrize(
    "args",
    [
        [
            "-v1",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
        ],
        [
            "-v2c",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
        ],
        # @todo: v3
    ],
)
def test_get(args: List[str], snmpd: Snmpd) -> None:
    cli = Cli()
    r = cli.run(args)
    assert r == ExitCode.OK


@pytest.mark.parametrize(
    ("fmt", "expected"),
    [
        ([], f"{SNMP_LOCATION_OID} = Gufo SNMP Test"),
        (["-Oa"], f"{SNMP_LOCATION_OID} = Gufo SNMP Test"),
        (
            ["-Ox"],
            f"{SNMP_LOCATION_OID} = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74",
        ),
        (
            ["-OT"],
            f"{SNMP_LOCATION_OID} = Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74",
        ),
        (["-Oq"], f"{SNMP_LOCATION_OID} Gufo SNMP Test"),
        (["-OQ"], f"{SNMP_LOCATION_OID} = Gufo SNMP Test"),
        (["-Ov"], "Gufo SNMP Test"),
        (["-Ovx"], "47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"),
        (["-Ov", "-Ox"], "47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"),
        (
            ["-Ov", "-OT"],
            "Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74",
        ),
    ],
)
def test_get_format(
    fmt: List[str], expected: str, capsys: pytest.CaptureFixture, snmpd: Snmpd
) -> None:
    cli = Cli()
    r = cli.run(
        [
            *fmt,
            "-v2c",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
        ]
    )
    captured = capsys.readouterr()
    out = captured.out.rstrip("\n")
    assert out == expected
    assert r == ExitCode.OK
