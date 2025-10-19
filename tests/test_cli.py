# ---------------------------------------------------------------------
# Gufo SNMP: Cli tests
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
from typing import Any, Dict, Iterable, List

# Third-party modules
import pytest

# Gufo SNMP modules
from gufo.snmp import SnmpVersion
from gufo.snmp.cli import Cli, ExitCode, Formatter, StrFormat, main
from gufo.snmp.snmpd import IS_DARWIN, Snmpd
from gufo.snmp.user import Aes128Key, DesKey, KeyType, Md5Key, Sha1Key

from .util import (
    SNMP_COMMUNITY,
    SNMP_CONTACT_OID,
    SNMP_LOCATION_OID,
    SNMP_SYSTEM_OID,
    SNMP_USERS,
    SNMPD_ADDRESS,
    SNMPD_PORT,
)

users = {u.name: u for u in SNMP_USERS}


def user_opts(name: str, oid: str) -> List[str]:
    """
    Generate command-line options for user.

    Args:
        name: user name.
        oid: Requested oid

    Returns:
        user authentication cli options.
    """
    user = users[name]
    r = ["-u", user.name]
    if user.auth_key:
        if user.auth_key.key_type != KeyType.Password:
            msg = f"User {name} auth key must be of password type"
            raise ValueError(msg)
        if user.auth_key.AUTH_ALG == Md5Key.AUTH_ALG:
            r += ["-a", "MD5"]
        elif user.auth_key.AUTH_ALG == Sha1Key.AUTH_ALG:
            r += ["-a", "SHA"]
        else:
            msg = f"Unknown auth protocol: {user.auth_key.AUTH_ALG}"
            raise ValueError(msg)
        r += ["-A", user.auth_key.key.decode()]
    if user.priv_key:
        if user.priv_key.key_type != KeyType.Password:
            msg = f"User {name} security key must be of password type"
            raise ValueError(msg)
        if user.priv_key.PRIV_ALG == DesKey.PRIV_ALG:
            r += ["-x", "DES"]
        elif user.priv_key.PRIV_ALG == Aes128Key.PRIV_ALG:
            r += ["-x", "AES"]
        else:
            msg = f"Unknown security protocol: {user.priv_key.AUTH_ALG}"
            raise ValueError(msg)
        r += ["-X", user.priv_key.key.decode()]
    # Common options
    r += ["-v3", "-p", str(SNMPD_PORT), SNMPD_ADDRESS, oid]
    return r


def test_die() -> None:
    with pytest.raises(SystemExit):
        Cli.die("die!")


@pytest.mark.parametrize(
    "args",
    [
        # Invalid version
        ["--version", "??", SNMPD_ADDRESS, "1.3.6"],
        # Mutually exclusive versions
        ["-v1", "-v2c", SNMPD_ADDRESS, "1.3.6"],
        # No community for v1/v2c
        [SNMPD_ADDRESS, "1.3.6"],
        ["-v1", SNMPD_ADDRESS, "1.3.6"],
        ["-v2c", SNMPD_ADDRESS, "1.3.6"],
        # -c for v3
        ["-v3", "-u", "user", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
        # v3 options for v2c
        ["-v2c", "-c", "public", "-a", "MD5", SNMPD_ADDRESS, "1.3.6"],
        ["-v2c", "-c", "public", "-A", "pass", SNMPD_ADDRESS, "1.3.6"],
        ["-v2c", "-c", "public", "-x", "AES", SNMPD_ADDRESS, "1.3.6"],
        ["-v2c", "-c", "public", "-X", "pass", SNMPD_ADDRESS, "1.3.6"],
        # No user for v3
        ["-v3", SNMPD_ADDRESS, "1.3.6"],
        # Invalid command
        ["--command", "GETWHAT", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
        # Invalid port
        ["-p", "AAA", SNMPD_ADDRESS, "1.3.6"],
        # Invalid OID
        ["-c", "public", SNMPD_ADDRESS, ".1.3.6"],
        # GETBULK for SNMPv1
        [
            "-v1",
            "--command",
            "GETBULK",
            "-c",
            "public",
            SNMPD_ADDRESS,
            "1.3.6",
        ],
        # Invalid format
        [
            "-v1",
            "-OX",
            "-c",
            "public",
            SNMPD_ADDRESS,
            "1.3.6",
        ],
        [
            "-v1",
            "-OXYZ",
            "-c",
            "public",
            SNMPD_ADDRESS,
            "1.3.6",
        ],
        # auth/no auth pass
        ["-v3", "-u", "test", "-a", "MD5", SNMPD_ADDRESS, "1.3.6"],
        # no auth/auth pass
        ["-v3", "-u", "test", "-A", "PASS", SNMPD_ADDRESS, "1.3.6"],
        # priv without auth
        ["-v3", "-u", "test", "-x", "AES", SNMPD_ADDRESS, "1.3.6"],
        # priv proto/no pass
        [
            "-v3",
            "-u",
            "test",
            "-a",
            "MD5",
            "-A",
            "PASS",
            "-x",
            "AES",
            SNMPD_ADDRESS,
            "1.3.6",
        ],
        # priv pass/no proto
        [
            "-v3",
            "-u",
            "test",
            "-a",
            "MD5",
            "-A",
            "PASS",
            "-X",
            "PASS",
            SNMPD_ADDRESS,
            "1.3.6",
        ],
    ],
)
def test_invalid_options(args: List[str]) -> None:
    with pytest.raises(SystemExit):
        main(args)


@pytest.mark.parametrize(
    ("args", "expected_str", "expected_version"),
    [
        # Default v2c
        (["-c", "public", SNMPD_ADDRESS, "1.3.6"], "v2c", SnmpVersion.v2c),
        # Short options
        (
            ["-v1", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
            "v1",
            SnmpVersion.v1,
        ),
        (
            ["-v2c", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
            "v2c",
            SnmpVersion.v2c,
        ),
        (["-v3", "-u", "user", SNMPD_ADDRESS, "1.3.6"], "v3", SnmpVersion.v3),
        # Long option
        (
            ["--version=v1", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
            "v1",
            SnmpVersion.v1,
        ),
        (
            ["--version=v2c", "-c", "public", SNMPD_ADDRESS, "1.3.6"],
            "v2c",
            SnmpVersion.v2c,
        ),
        (
            ["--version=v3", "-u", "user", SNMPD_ADDRESS, "1.3.6"],
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
        (["-c", "public", SNMPD_ADDRESS, "1.3.6"], 161),
        (["-c", "public", "-p", "10000", SNMPD_ADDRESS, "1.3.6"], 10000),
        (["-c", "public", "--port=10000", SNMPD_ADDRESS, "1.3.6"], 10000),
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
    ("cfg", "v", "expected"),
    [
        # None
        ({}, None, "null"),
        # int
        ({}, 42, "42"),
        # float
        ({}, 1.2, "1.2"),
        # str
        ({}, "1.3.6", "1.3.6"),
        # ascii
        ({"str_format": StrFormat.ASCII}, b"test value\n", "test value."),
        (
            {"str_format": StrFormat.HEX},
            b"test value\n",
            "74 65 73 74 20 76 61 6C 75 65 0A",
        ),
        (
            {"str_format": StrFormat.ASCII_HEX},
            b"test value\n",
            "test value. 74 65 73 74 20 76 61 6C 75 65 0A",
        ),
        (
            {"str_format": StrFormat.REPR},
            b"test value\n",
            "b'test value\\n'",
        ),
    ],
    ids=[
        "null",
        "int",
        "float",
        "oid",
        "str-ascii",
        "str-hex",
        "str-ascii-hex",
        "str-repr",
    ],
)
def test_format_value(cfg: Dict[str, Any], v: Any, expected: str) -> None:
    formatter = Formatter(**cfg)
    r = formatter.format_value(v)
    assert r == expected


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
        # v3
        user_opts("user0000", SNMP_LOCATION_OID),
        user_opts("user1000", SNMP_LOCATION_OID),
        user_opts("user1010", SNMP_LOCATION_OID),
        user_opts("user1020", SNMP_LOCATION_OID),
        user_opts("user2000", SNMP_LOCATION_OID),
        user_opts("user2010", SNMP_LOCATION_OID),
        user_opts("user2020", SNMP_LOCATION_OID),
    ],
    ids=[
        "v1",
        "v2c",
        "v3",
        "v3-md5",
        "v3-md5-des",
        "v3-md5-aes128",
        "v3-sha1",
        "v3-sha1-des",
        "v3-sha1-aes128",
    ],
)
def test_get(args: List[str], snmpd: Snmpd) -> None:
    r = main(args)
    assert r == ExitCode.OK.value


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
    ids=["default", "a", "x", "T", "q", "Q", "v", "vx", "v-x", "v-T"],
)
def test_get_format(
    fmt: List[str], expected: str, capsys: pytest.CaptureFixture, snmpd: Snmpd
) -> None:
    r = main(
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
    assert r == ExitCode.OK.value


@pytest.mark.parametrize(
    ("fmt", "expected"),
    [
        (
            [],
            (
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n"
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test"
            ),
        ),
        (
            ["-Oa"],
            (
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n"
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test"
            ),
        ),
        (
            ["-Ox"],
            (
                f"{SNMP_CONTACT_OID} = 74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n"
                f"{SNMP_LOCATION_OID} = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"
            ),
        ),
        (
            ["-OT"],
            (
                f"{SNMP_CONTACT_OID} = test <test@example.com> 74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n"
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"
            ),
        ),
        (
            ["-Oq"],
            (
                f"{SNMP_CONTACT_OID} test <test@example.com>\n"
                f"{SNMP_LOCATION_OID} Gufo SNMP Test"
            ),
        ),
        (
            ["-OQ"],
            (
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n"
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test"
            ),
        ),
        (["-Ov"], "test <test@example.com>\nGufo SNMP Test"),
        (
            ["-Ovx"],
            (
                "74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n"
                "47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"
            ),
        ),
        (
            ["-Ov", "-Ox"],
            (
                "74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n"
                "47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"
            ),
        ),
        (
            ["-Ov", "-OT"],
            (
                "test <test@example.com> 74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n"
                "Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74"
            ),
        ),
    ],
    ids=["default", "a", "x", "T", "q", "Q", "v", "vx", "v-x", "v-T"],
)
def test_get_many_format(
    fmt: List[str], expected: str, capsys: pytest.CaptureFixture, snmpd: Snmpd
) -> None:
    r = main(
        [
            *fmt,
            "-v2c",
            "-c",
            SNMP_COMMUNITY,
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
            SNMP_CONTACT_OID,
        ]
    )
    captured = capsys.readouterr()
    out = captured.out.rstrip("\n")
    assert out == expected
    assert r == ExitCode.OK.value


@pytest.mark.parametrize(
    ("args", "expected"),
    [
        # GETNEXT
        (
            [
                "-v1",
                "--command",
                "GETNEXT",
                "-c",
                SNMP_COMMUNITY,
            ],
            [
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n",
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test\n",
            ],
        ),
        (
            [
                "-v2c",
                "--command",
                "GETNEXT",
                "-c",
                SNMP_COMMUNITY,
            ],
            [
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n",
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test\n",
            ],
        ),
        (
            [
                "-v2c",
                "--command",
                "GETNEXT",
                "-Ox",
                "-c",
                SNMP_COMMUNITY,
            ],
            [
                f"{SNMP_CONTACT_OID} = 74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n",
                f"{SNMP_LOCATION_OID} = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74\n",
            ],
        ),
        # GETBULK
        (
            [
                "-v2c",
                "--command",
                "GETBULK",
                "-c",
                SNMP_COMMUNITY,
            ],
            [
                f"{SNMP_CONTACT_OID} = test <test@example.com>\n",
                f"{SNMP_LOCATION_OID} = Gufo SNMP Test\n",
            ],
        ),
        (
            [
                "-v2c",
                "--command",
                "GETBULK",
                "-Ox",
                "-c",
                SNMP_COMMUNITY,
            ],
            [
                f"{SNMP_CONTACT_OID} = 74 65 73 74 20 3C 74 65 73 74 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 3E\n",
                f"{SNMP_LOCATION_OID} = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74\n",
            ],
        ),
    ],
    ids=[
        "v1-GETNEXT",
        "v2c-GETNEXT",
        "v2c-x-GETNEXT",
        "v2c-GETBULK",
        "v2c-x-GETBULK",
    ],
)
def test_get_table(
    args: List[str],
    expected: Iterable[str],
    capsys: pytest.CaptureFixture,
    snmpd: Snmpd,
) -> None:
    r = main([*args, "-p", str(SNMPD_PORT), SNMPD_ADDRESS, SNMP_SYSTEM_OID])
    captured = capsys.readouterr()
    out = captured.out.rstrip("\n")
    for x in expected:
        assert x in out
    assert r == ExitCode.OK.value


# @todo: v2c tests fail on MacOS, temporary skip them
@pytest.mark.parametrize(
    "args",
    [
        # GET
        pytest.param(
            (
                "-v2c",
                "--command",
                "GET",
                "-c",
                "invalid",
                "-p",
                str(SNMPD_PORT),
                SNMPD_ADDRESS,
                SNMP_LOCATION_OID,
            ),
            marks=pytest.mark.skipif(
                IS_DARWIN, reason="Temporary disabled on MacOS"
            ),
        ),
        (
            "-v3",
            "--command",
            "GET",
            "-u",
            "invalid",
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
        ),
        # GET MANY
        pytest.param(
            (
                "-v2c",
                "--command",
                "GET",
                "-c",
                "invalid",
                "-p",
                str(SNMPD_PORT),
                SNMPD_ADDRESS,
                SNMP_LOCATION_OID,
                SNMP_CONTACT_OID,
            ),
            marks=pytest.mark.skipif(
                IS_DARWIN, reason="Temporary disabled on MacOS"
            ),
        ),
        (
            "-v3",
            "--command",
            "GET",
            "-u",
            "invalid",
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_LOCATION_OID,
            SNMP_CONTACT_OID,
        ),
        # GETNEXT
        pytest.param(
            (
                "-v2c",
                "--command",
                "GETNEXT",
                "-c",
                "invalid",
                "-p",
                str(SNMPD_PORT),
                SNMPD_ADDRESS,
                SNMP_SYSTEM_OID,
            ),
            marks=pytest.mark.skipif(
                IS_DARWIN, reason="Temporary disabled on MacOS"
            ),
        ),
        (
            "-v3",
            "--command",
            "GETNEXT",
            "-u",
            "invalid",
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_SYSTEM_OID,
        ),
        # GETBULK
        pytest.param(
            (
                "-v2c",
                "--command",
                "GETBULK",
                "-c",
                "invalid",
                "-p",
                str(SNMPD_PORT),
                SNMPD_ADDRESS,
                SNMP_SYSTEM_OID,
            ),
            marks=pytest.mark.skipif(
                IS_DARWIN, reason="Temporary disabled on MacOS"
            ),
        ),
        (
            "-v3",
            "--command",
            "GETBULK",
            "-u",
            "invalid",
            "-p",
            str(SNMPD_PORT),
            SNMPD_ADDRESS,
            SNMP_SYSTEM_OID,
        ),
    ],
    ids=[
        "get-v2c",
        "get-v3",
        "getmany-v2c",
        "getmany-v3",
        "getnext-v2c",
        "getnext-v3",
        "getbulk-v2c",
        "getbulk-v3",
    ],
)
def test_auth_error(args: List[str], snmpd: Snmpd) -> None:
    with pytest.raises(SystemExit):
        main(args)
