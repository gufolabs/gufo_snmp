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
from gufo.snmp.cli import Cli, ExitCode
from gufo.snmp.snmpd import Snmpd

from .util import SNMP_COMMUNITY, SNMPD_ADDRESS, SNMPD_PORT


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
    ],
)
def test_invalid_options(args: List[str]) -> None:
    with pytest.raises(SystemExit):
        Cli().run(args)


@pytest.mark.parametrize(
    ("args", "expected"),
    [
        # Default v2c
        (["-c", "public", "127.0.0.1", "1.3.6"], "v2c"),
        # Short options
        (["-v1", "-c", "public", "127.0.0.1", "1.3.6"], "v1"),
        (["-v2c", "-c", "public", "127.0.0.1", "1.3.6"], "v2c"),
        (["-v3", "-u", "user", "127.0.0.1", "1.3.6"], "v3"),
        # Long option
        (["--version=v1", "-c", "public", "127.0.0.1", "1.3.6"], "v1"),
        (["--version=v2c", "-c", "public", "127.0.0.1", "1.3.6"], "v2c"),
        (["--version=v3", "-u", "user", "127.0.0.1", "1.3.6"], "v3"),
    ],
)
def test_parse_version(args: List[str], expected: str) -> None:
    ns = Cli.parse_args(args)
    assert ns.version and ns.version == expected


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
    assert ns.port and ns.port == expected


@pytest.mark.parametrize(
    "args",
    [
        (
            "-v1",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            "1.3.6.1.2.1.1",
        ),
        (
            "-v2c",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            "1.3.6.1.2.1.1",
        ),
    ],
)
def test_get(args: List[str], snmpd: Snmpd) -> None:
    cli = Cli()
    r = cli.run(args)
    assert r == ExitCode.OK
