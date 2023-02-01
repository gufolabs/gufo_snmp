# ---------------------------------------------------------------------
# Gufo SNMP: Snmpd context manager
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""snmpd context manager."""

# Python modules
import subprocess
import threading
from tempfile import NamedTemporaryFile, _TemporaryFileWrapper
from types import TracebackType
from typing import List, Optional, Type


class Snmpd(object):
    """
    snmpd context manager for testing.

    The context manager running snmpd instance
    for testing purposes. Requires Net-SNMP
    to be installed.

    Args:
        path: snmpd path.
        address: Address to listen.
        port: Port to listen.
        community: SNMP v1/v2c community.
        location: sysLocation value.
        contact: sysContact value.
        user: SNMP v3 user.
        start_timeout: Maximum time to wait for snmpd to start.
        dump_log: Dump snmpd log on premature exit.

    Attributes:
        version: Net-SNMP version.

    Note:
        Using the ports below 1024 usually requires
        the root priveleges.

    Example:
        ``` py
        with Snmpd():
            # Any Gufo SNMP code
        ```

    Example:
        ``` py
        async with Snmpd():
            # Any Gufo SNMP code
        ```
    """

    def __init__(
        self: "Snmpd",
        path: str = "/usr/sbin/snmpd",
        address: str = "127.0.0.1",
        port: int = 10161,
        community: str = "public",
        location: str = "Test",
        contact: str = "test <test@example.com>",
        user: str = "rouser",
        start_timeout: float = 5.0,
        dump_log: bool = False,
    ) -> None:
        self._path = path
        self._address = address
        self._port = port
        self._community = community
        self._location = location
        self._contact = contact
        self._user = user
        self._start_timeout = start_timeout
        self._dump_log = dump_log
        self.version: Optional[str] = None
        self._cfg: Optional[_TemporaryFileWrapper[str]] = None
        self._proc: Optional[subprocess.Popen[str]] = None

    def get_config(self: "Snmpd") -> str:
        """
        Generate snmpd config.

        Returns:
            snmpd configuration.
        """
        return f"""# Gufo SNMP Test Suite
master agentx
agentaddress udp:{self._address}:{self._port}
# Listen address
# SNMPv1/SNMPv2c R/O community
rocommunity {self._community} 127.0.0.1
# SNMPv3 R/O User
rouser {self._user} auth
# System information
syslocation {self._location}
syscontact  {self._contact}
#
sysServices 72"""

    def _start(self: "Snmpd") -> None:
        """Run snmpd instance."""
        self._cfg = NamedTemporaryFile(
            prefix="snmpd-", suffix=".conf", mode="w"
        )
        self._cfg.write(self.get_config())
        # Ensure the file is written
        self._cfg.flush()
        # Run snmpd
        self._proc = subprocess.Popen(
            [
                self._path,
                "-C",  # Ignore default configs
                "-c",  # Read our config
                self._cfg.name,
                "-f",  # No fork
                "-Lo",  # Log to stdout
                "-V",  # Verbose
                "-d",  # Dump packets
            ],
            stdout=subprocess.PIPE,
            encoding="utf-8",
            text=True,
        )
        # Wait for snmpd is up
        self._wait()
        self._consume_stdout()

    def _wait(self: "Snmpd") -> None:
        """Wait until snmpd is ready."""

        def inner() -> None:
            if self._proc and self._proc.stdout:
                log: List[str] = []
                for line in self._proc.stdout:
                    if self._dump_log:
                        log.append(line)
                    if line.startswith("NET-SNMP version"):
                        self.version = line.strip().split(" ", 2)[2].strip()
                        return
                # Premature termination of snmpd
                if self._dump_log:
                    print("".join(log))
                msg = "snmpd is terminated prematurely"
                raise RuntimeError(msg)
            msg = "snmpd is not active"
            raise RuntimeError(msg)

        if self._proc is None:
            msg = "_wait() must not be started directly"
            raise RuntimeError(msg)
        if not self._proc.stdout:
            msg = "stdout is not piped"
            raise RuntimeError(msg)
        t = threading.Thread(target=inner)
        t.daemon = True
        t.start()
        t.join(self._start_timeout)
        if t.is_alive():
            msg = "snmpd failed to start"
            raise TimeoutError(msg)

    def _consume_stdout(self: "Snmpd") -> None:
        def inner() -> None:
            if self._proc and self._proc.stdout:
                for _ in self._proc.stdout:
                    pass

        t = threading.Thread(target=inner)
        t.daemon = True
        t.start()

    def _stop(self: "Snmpd") -> None:
        """Terminate snmpd instance."""
        if self._proc:
            self._proc.kill()
        if self._cfg:
            self._cfg.close()

    def __enter__(self: "Snmpd") -> "Snmpd":
        """Context manager entry."""
        self._start()
        return self

    def __exit__(
        self: "Snmpd",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Context manager exit."""
        self._stop()

    async def __aenter__(self: "Snmpd") -> "Snmpd":
        """Asynchronous context manager entry."""
        self._start()
        return self

    async def __aexit__(
        self: "Snmpd",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Asynchronous context manager exit."""
        self._stop()
