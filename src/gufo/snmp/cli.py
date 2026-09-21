# ---------------------------------------------------------------------
# Gufo SNMP: Cli command
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""
gufo-snmp command.

Attributes:
    NAME: Utility's name.
"""

# Python modules
from __future__ import annotations

import argparse
import re
import sys
from collections.abc import Callable, Sequence
from enum import Enum, IntEnum
from operator import itemgetter
from typing import Any, NoReturn, cast

# Gufo SNMP modules
from gufo.snmp import (
    Aes128Key,
    Aes192Key,
    Aes256Key,
    BaseAuthKey,
    BasePrivKey,
    DesKey,
    Md5Key,
    Sha1Key,
    Sha224Key,
    Sha256Key,
    Sha384Key,
    Sha512Key,
    SnmpAuthError,
    SnmpVersion,
    User,
    ValueType,
)
from gufo.snmp.sync import SnmpSession

NAME = "gufo-snmp"


class ExitCode(IntEnum):
    """
    Cli exit codes.

    Attributes:
        OK: Successful exit.
        ERR: Error.
    """

    OK = 0
    ERR = 1


class Command(Enum):
    """Command to execute."""

    GET = "get"
    GETMANY = "getmany"
    GETNEXT = "getnext"
    GETBULK = "getbulk"


VERSION_MAP = {
    "v1": SnmpVersion.v1,
    "v2c": SnmpVersion.v2c,
    "v3": SnmpVersion.v3,
}

DEFAULT_SEP = " = "


class StrFormat(Enum):
    """
    OctetString formatters.

    Attributes:
        REPR: Using python's repr().
        ASCII: Show printable ASCII symbols, replacing others with dots.
        HEX: Hex-dump.
        ASCII_HEX: ASCII + HEX
    """

    REPR = "repr"
    ASCII = "ascii"
    HEX = "hex"
    ASCII_HEX = "asciihex"

    @classmethod
    def default(cls) -> StrFormat:
        """Get the default string format.

        Returns:
            The default string format.
        """
        return cls.ASCII


MIN_PRINTABLE = 0x20
MAX_PRINTABLE = 0x7F


class Formatter:
    """Pretty format output."""

    def __init__(
        self,
        /,
        show_key: bool = True,
        sep: str = DEFAULT_SEP,
        str_format: StrFormat = StrFormat.ASCII,
    ) -> None:
        self.show_key = show_key
        self.sep = sep
        self._format_str: Callable[[bytes], str] = getattr(
            self, f"_format_str_{str_format.value}"
        )

    @classmethod
    def validate(cls, parser: argparse.ArgumentParser, opts: str) -> None:
        """Validate output formatting options.

        Args:
            parser: Argument parser used to report validation errors.
            opts: Output formatting options.
        """
        if not opts:
            return
        invalid = list(set(opts) - set(OFLAGS_HELP))
        if len(invalid) == 1:
            parser.error(f"Invalid format option: {invalid[0]}")
        elif len(invalid) > 1:
            parser.error(f"Invalid format options: {', '.join(invalid)}")

    @classmethod
    def from_opts(cls, opts: str) -> Formatter:
        """Build a formatter from output formatting options.

        Args:
            opts: Output formatting options.

        Returns:
            Configured formatter.
        """
        show_key = True
        sep = DEFAULT_SEP
        str_format = StrFormat.default()
        for opt in opts:
            match opt:
                case "a":
                    str_format = StrFormat.ASCII
                case "x":
                    str_format = StrFormat.HEX
                case "q":
                    sep = " "
                case "Q":
                    sep = " = "
                case "T":
                    str_format = StrFormat.ASCII_HEX
                case "v":
                    show_key = False
                    sep = ""
                case _:
                    pass
        return Formatter(
            show_key=show_key,
            sep=sep,
            str_format=str_format,
        )

    def format_value(self, value: ValueType) -> str:
        """Format a value.

        Args:
            value: Value to format.

        Returns:
            Formatted value.
        """
        match value:
            case None:
                return "null"
            case int() | float():
                return str(value)
            case str():
                return value  # OID
            case bytes():
                return self._format_str(value)
            case _:
                return self._format_str_repr(value)

    def format(self, oid: str, value: ValueType) -> str:
        """Format an OID and its value.

        Args:
            oid: Object identifier.
            value: Value to format.

        Returns:
            Formatted OID and value.
        """
        v = self.format_value(value)
        return f"{oid if self.show_key else ''}{self.sep}{v}"

    @staticmethod
    def _format_str_ascii(s: bytes) -> str:
        """Format bytes as printable ASCII.

        Args:
            s: Bytes to format.

        Returns:
            ASCII representation with non-printable bytes replaced by dots.
        """
        return "".join(
            chr(b) if MIN_PRINTABLE <= b < MAX_PRINTABLE else "." for b in s
        )

    @staticmethod
    def _format_str_asciihex(s: bytes) -> str:
        """Format bytes as ASCII and hexadecimal.

        Args:
            s: Bytes to format.

        Returns:
            ASCII representation followed by a hexadecimal representation.
        """
        a = Formatter._format_str_ascii(s)
        x = Formatter._format_str_hex(s)
        return f"{a} {x}"

    @staticmethod
    def _format_str_hex(s: bytes) -> str:
        """Format bytes as hexadecimal.

        Args:
            s: Bytes to format.

        Returns:
            Space-separated hexadecimal representation.
        """
        return " ".join(f"{b:02X}" for b in s)

    @staticmethod
    def _format_str_repr(s: bytes) -> str:
        """Format bytes using ``repr()``.

        Args:
            s: Bytes to format.

        Returns:
            Python representation of the bytes.
        """
        return repr(s)


class CollectOFlags(argparse.Action):
    """Collect flags from a single ``-O`` option occurrence.

    Args:
        parser: Argument parser invoking this action.
        namespace: Namespace where the parsed flags are stored.
        values: Flag characters passed to ``-O``.
        option_string: Option string that triggered this action.
    """

    def __call__(
        self,
        parser: argparse.ArgumentParser,
        namespace: argparse.Namespace,
        values: str | Sequence[Any] | None,
        option_string: str | None = None,
    ) -> None:
        """
        Process a single -O argument occurrence.

        Each time the -O option is encountered, this method extracts the
        individual flag characters from *values* and merges them into the
        destination attribute (a set of characters). Duplicate flags are
        ignored automatically.

        Args:
            parser: The argument parser invoking this action.
            namespace: The namespace object where parsed values are stored.
            values: The string of flag characters passed to -O
                (e.g., "nqv" for `-Onqv`).
            option_string: The option string that triggered this action,
                e.g., "-O" (may be None when called programmatically).
        """
        flags: set[str] = getattr(namespace, self.dest, set()) or set()
        flags.update(values or ())
        setattr(namespace, self.dest, flags)


OFLAGS_HELP = {
    "a": "print all strings in ascii format",
    "x": "print all strings in hex format",
    "q": "quick print for easier parsing",
    "Q": "quick print with equal-signs",
    "T": "print human-readable text along with hex strings",
    "v": "print values only (not OID = value)",
}

AUTH_PROTOCOL: dict[str, type[BaseAuthKey]] = {
    "MD5": Md5Key,
    "SHA": Sha1Key,
    "SHA224": Sha224Key,
    "SHA256": Sha256Key,
    "SHA384": Sha384Key,
    "SHA512": Sha512Key,
}
PRIV_PROTOCOL: dict[str, type[BasePrivKey]] = {
    "DES": DesKey,
    "AES": Aes128Key,
    "AES128": Aes128Key,
    "AES192": Aes192Key,
    "AES256": Aes256Key,
}


class Cli:
    """`gufo-snmp` utility class."""

    @classmethod
    def die(cls, msg: str | None = None) -> NoReturn:
        """Print an error message and terminate the process.

        Args:
            msg: Error message to print.
        """
        if msg:
            print(msg)
        sys.exit(1)

    @classmethod
    def parse_args(cls, args: list[str]) -> argparse.Namespace:
        """Parse and validate command-line arguments.

        Args:
            args: Command-line arguments.

        Returns:
            Parsed and validated arguments.
        """
        # Prepare parser
        parser = argparse.ArgumentParser(
            prog=NAME,
            description="SNMP Client",
            formatter_class=argparse.RawTextHelpFormatter,
        )
        parser.add_argument("address", nargs=1, help="Agent")
        parser.add_argument("oids", nargs=argparse.REMAINDER, help="OIDs")
        # Protocol version
        parser.add_argument(
            "--version",
            type=str,
            choices=list(VERSION_MAP),
            default="v2c",
            help="SNMP Protocol version",
        )
        version_group = parser.add_mutually_exclusive_group()
        version_group.add_argument(
            "-v1",
            dest="version",
            action="store_const",
            const="v1",
            help="SNMP v1",
        )
        version_group.add_argument(
            "-v2c",
            dest="version",
            action="store_const",
            const="v2c",
            help="SNMP v2c",
        )
        version_group.add_argument(
            "-v3",
            dest="version",
            action="store_const",
            const="v3",
            help="SNMP v3",
        )
        # Command
        parser.add_argument(
            "--command",
            default="GET",
            choices=["GET", "GETNEXT", "GETBULK"],
            help="Command",
        )
        parser.add_argument(
            "-p", "--port", type=int, default=161, help="Argent port"
        )
        # v2c
        parser.add_argument("-c", "--community", help="Community (v1/v2c)")
        # v3
        parser.add_argument("-u", "--user", help="User name (v3)")
        parser.add_argument(
            "-a",
            "--auth-protocol",
            choices=list(AUTH_PROTOCOL),
            help="Set authentication protocol (v3)",
        )
        parser.add_argument(
            "-A",
            "--auth-pass",
            help="Set authentication protocol pass-phrase (v3)",
        )
        parser.add_argument(
            "-x",
            "--security-protocol",
            choices=list(PRIV_PROTOCOL),
            help="Set security protocol (v3)",
        )
        parser.add_argument(
            "-X",
            "--security-pass",
            help="Set security protocol pass-phrase (v3)",
        )
        # Format
        parser.add_argument(
            "-O",
            action=CollectOFlags,
            dest="oflags",
            help=(
                "Output formatting flags (may be repeated or combined)\n"
                "Supported flags:\n  "
                + "\n  ".join(f"{k} : {v}" for k, v in OFLAGS_HELP.items())
            ),
        )
        # Parse arguments
        ns = parser.parse_args(args)
        # @todo: validate agent address
        # validate oids
        for oid in ns.oids:
            if not cls.is_valid_oid(oid):
                parser.error(f"Invalid OID: {oid}")
        # Additional checks
        Formatter.validate(parser, ns.oflags)
        if ns.version == "v1" and ns.command == "GETBULK":
            parser.error("GETBULK is not defined for SNMPv1")
        if ns.version in ("v1", "v2c"):
            cls._validate_community(parser, ns)
        elif ns.version == "v3":
            cls._validate_usm(parser, ns)
        # Validated
        return ns

    rx_oid = re.compile(r"^1.3.6(\.\d+)*$")

    @classmethod
    def is_valid_oid(cls, oid: str) -> bool:
        """Check whether an OID is valid.

        Args:
            oid: Object identifier to validate.

        Returns:
            ``True`` if the OID is valid, otherwise ``False``.
        """
        return bool(cls.rx_oid.match(oid))

    @classmethod
    def _validate_community(
        cls, parser: argparse.ArgumentParser, ns: argparse.Namespace
    ) -> None:
        """Validate community-based security options.

        Args:
            parser: Argument parser used to report validation errors.
            ns: Parsed command-line arguments.
        """
        if not ns.community:
            parser.error(f"SNMP {ns.version} requires -c/--community")
        for v, opt in (
            (ns.auth_protocol, "-A/--auth-protocol"),
            (ns.auth_pass, "-a/--auth-pass"),
            (ns.security_protocol, "-x/--security-protocol"),
            (ns.security_pass, "-X/--security-pass"),
        ):
            if v:
                parser.error(f"SNMP {ns.version} doesn't support {opt} option")

    @classmethod
    def _validate_usm(
        cls, parser: argparse.ArgumentParser, ns: argparse.Namespace
    ) -> None:
        """Validate USM security options.

        Args:
            parser: Argument parser used to report validation errors.
            ns: Parsed command-line arguments.
        """
        if not ns.user:
            parser.error(f"SNMP {ns.version} requires -u/--user")
        # Auth
        if ns.auth_protocol and not ns.auth_pass:
            parser.error("-A/--auth-pass is required for -a/--auth-protocol")
        if ns.auth_pass and not ns.auth_protocol:
            parser.error("-a/--auth-protocol is required for -A/--auth-pass")
        # Priv
        if ns.security_protocol and not ns.security_pass:
            parser.error(
                "-X/--security-pass is required for -x/--security-protocol"
            )
        if not ns.security_protocol and ns.security_pass:
            parser.error(
                "-x/--security-protocol is required for -X/--security-pass"
            )
        # Auth must be set for priv
        if ns.security_protocol and not ns.auth_protocol:
            parser.error(
                "-a/--auth-protocol must be set for -x/--security-protocol"
            )
        if ns.community:
            parser.error(
                f"SNMP {ns.version} doesn't support -c/--community option"
            )

    def get_version(self, ns: argparse.Namespace) -> SnmpVersion:
        """Get the SNMP version from command-line arguments.

        Args:
            ns: Parsed command-line arguments.

        Returns:
            SNMP protocol version.
        """
        return VERSION_MAP[ns.version]

    def get_command(self, ns: argparse.Namespace) -> Command:
        """Determine the command to execute.

        Args:
            ns: Parsed command-line arguments.

        Returns:
            Command to execute.
        """
        match ns.command:
            case "GET" if len(ns.oids) == 1:
                return Command.GET
            case "GETNEXT":
                return Command.GETNEXT
            case "GETBULK":
                return Command.GETBULK
            case _:
                return Command.GETMANY

    def get_community(self, ns: argparse.Namespace) -> str:
        """Get the SNMP community from command-line arguments.

        Args:
            ns: Parsed command-line arguments.

        Returns:
            SNMP community string.
        """
        return cast(str, ns.community)

    def get_user(self, ns: argparse.Namespace) -> User:
        """Build the USM user configuration from command-line arguments.

        Args:
            ns: Parsed command-line arguments.

        Returns:
            Configured USM user.
        """
        # Process auth key
        auth_key = None
        if ns.auth_pass:
            auth_key = AUTH_PROTOCOL[ns.auth_protocol](ns.auth_pass.encode())
        # Process priv key
        priv_key = None
        if auth_key and ns.security_pass:
            priv_key = PRIV_PROTOCOL[ns.security_protocol](
                ns.security_pass.encode()
            )
        return User(ns.user, auth_key=auth_key, priv_key=priv_key)

    def get_session(self, ns: argparse.Namespace) -> SnmpSession:
        """Build an SNMP session from command-line arguments.

        Args:
            ns: Parsed command-line arguments.

        Returns:
            Configured SNMP session.
        """
        version = self.get_version(ns)
        community = (
            self.get_community(ns)
            if version in (SnmpVersion.v1, SnmpVersion.v2c)
            else ""
        )
        user = self.get_user(ns) if version == SnmpVersion.v3 else None
        return SnmpSession(
            addr=ns.address[0],
            port=ns.port,
            version=version,
            community=community,
            user=user,
            timeout=3.0,
        )

    def run(self, args: list[str]) -> ExitCode:
        """Parse arguments and run the appropriate command.

        Args:
            args: Command-line arguments.

        Returns:
            Command exit code.
        """
        ns = self.parse_args(args)
        cmd = self.get_command(ns)
        formatter = Formatter.from_opts(ns.oflags or "")
        try:
            with self.get_session(ns) as session:
                match cmd:
                    case Command.GET:
                        return self.run_get(session, ns.oids, formatter)
                    case Command.GETMANY:
                        return self.run_get_many(session, ns.oids, formatter)
                    case Command.GETNEXT:
                        return self.run_getnext(session, ns.oids, formatter)
                    case Command.GETBULK:
                        return self.run_getbulk(session, ns.oids, formatter)
                    case _:
                        return ExitCode.ERR
        except TimeoutError:
            self.die("ERROR: Timed out")
        except SnmpAuthError:
            self.die("ERROR: Authentication failed")

    def run_get(
        self, session: SnmpSession, oids: list[str], formatter: Formatter
    ) -> ExitCode:
        """Perform a GET request.

        Args:
            session: Configured SNMP session.
            oids: List of OIDs. Exactly one OID is expected.
            formatter: Formatter used to format the response.

        Returns:
            Command exit code.
        """
        oid = oids[0]
        r = session.get(oid)
        print(formatter.format(oid, r))
        return ExitCode.OK

    def run_get_many(
        self, session: SnmpSession, oids: list[str], formatter: Formatter
    ) -> ExitCode:
        """Perform a GET request for multiple OIDs.

        Args:
            session: Configured SNMP session.
            oids: OIDs to request.
            formatter: Formatter used to format the responses.

        Returns:
            Command exit code.
        """
        r = session.get_many(oids)
        for k, v in sorted(r.items(), key=itemgetter(0)):
            print(formatter.format(k, v))
        return ExitCode.OK

    def run_getnext(
        self, session: SnmpSession, oids: list[str], formatter: Formatter
    ) -> ExitCode:
        """Perform GETNEXT requests.

        Args:
            session: Configured SNMP session.
            oids: OIDs to use as starting points.
            formatter: Formatter used to format the responses.

        Returns:
            Command exit code.
        """
        for oid in oids:
            for k, v in session.getnext(oid):
                print(formatter.format(k, v))
        return ExitCode.OK

    def run_getbulk(
        self, session: SnmpSession, oids: list[str], formatter: Formatter
    ) -> ExitCode:
        """Perform GETBULK requests.

        Args:
            session: Configured SNMP session.
            oids: OIDs to use as starting points.
            formatter: Formatter used to format the responses.

        Returns:
            Command exit code.
        """
        for oid in oids:
            for k, v in session.getbulk(oid):
                print(formatter.format(k, v))
        return ExitCode.OK


def main(args: list[str] | None = None) -> int:
    """Run ``gufo-snmp`` with command-line arguments.

    Args:
        args: Command-line arguments. If ``None``, use ``sys.argv``.

    Returns:
        Process exit code.
    """
    return Cli().run(sys.argv[1:] if args is None else args).value
