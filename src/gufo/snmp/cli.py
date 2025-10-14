# ---------------------------------------------------------------------
# Gufo SNMP: Cli command
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""
gufo-snmp command.

Attributes:
    NAME: Utility's name.
"""

# Python modules
import argparse
import sys
from enum import IntEnum
from typing import List, NoReturn, Optional

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


class Cli(object):
    """`gufo-snmp` utility class."""

    @classmethod
    def die(cls, msg: Optional[str] = None) -> NoReturn:
        """Die with message."""
        if msg:
            print(msg)
        sys.exit(1)

    @classmethod
    def parse_args(cls, args: List[str]) -> argparse.Namespace:
        """
        Parse arguments.

        Args:
            args: Arguments list.

        Returns:
            Parsed namespace.
        """
        # Prepare parser
        parser = argparse.ArgumentParser(prog=NAME, description="SNMP Client")
        parser.add_argument("address", nargs=1, help="Agent")
        parser.add_argument("oids", nargs=argparse.REMAINDER, help="OIDs")
        # Protocol version
        parser.add_argument(
            "--version",
            type=str,
            choices=["v1", "v2c", "v3"],
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
            "-X",
            "--command",
            default="GET",
            choices=["GET", "GETNEXT", "GETBULK"],
            help="Command",
        )
        parser.add_argument(
            "-p", "--port", type=int, default=161, help="Argent port"
        )
        # Auth
        parser.add_argument("-c", "--community", help="Community (v1/v2c)")
        parser.add_argument("-u", "--user", help="User name (v3)")
        # Parse arguments
        ns = parser.parse_args(args)
        # Additional checks
        if ns.version in ("v1", "v2c"):
            cls._validate_community(parser, ns)
        elif ns.version == "v3":
            cls._validate_usm(parser, ns)
        # Validated
        return ns

    @classmethod
    def _validate_community(
        cls, parser: argparse.ArgumentParser, ns: argparse.Namespace
    ) -> None:
        """
        Validate community-based security options.

        Args:
            parser: Argument parser.
            ns: Parsed namespace.
        """
        if not ns.community:
            parser.error(f"SNMP {ns.version} requires -c/--community")

    @classmethod
    def _validate_usm(
        cls, parser: argparse.ArgumentParser, ns: argparse.Namespace
    ) -> None:
        """
        Validate USM security options.

        Args:
            parser: Argument parser.
            ns: Parsed namespace.
        """
        if not ns.user:
            parser.error(f"SNMP {ns.version} requires -u/--user")

    def run(self, args: List[str]) -> ExitCode:
        """
        Parse command-line arguments and run appropriative command.

        Args:
            args: List of command-line arguments
        Returns:
            ExitCode
        """
        ns = self.parse_args(args)
        return ExitCode.OK


def main(args: Optional[List[str]] = None) -> int:
    """Run `gufo-ping` with command-line arguments."""
    return Cli().run(sys.argv[1:] if args is None else args).value
