# ---------------------------------------------------------------------
# Gufo SNMP: Snmpd context manager
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""snmpd context manager."""

# Python modules
import logging
import queue
import subprocess
import threading
from tempfile import NamedTemporaryFile, _TemporaryFileWrapper
from types import TracebackType
from typing import Optional, Type

logger = logging.getLogger("gufo.snmp.snmpd")


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
        log_packets: Log SNMP requests and responses.

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
        log_packets: bool = False,
    ) -> None:
        self._path = path
        self._address = address
        self._port = port
        self._community = community
        self._location = location
        self._contact = contact
        self._user = user
        self._start_timeout = start_timeout
        self._log_packets = log_packets
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
agentXsocket tcp:{self._address}:{self._port}
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
        logger.info("Starting snmpd instance")
        self._cfg = NamedTemporaryFile(
            prefix="snmpd-", suffix=".conf", mode="w"
        )
        cfg = self.get_config()
        logger.debug("snmpd config:\n%s", cfg)
        self._cfg.write(cfg)
        # Ensure the file is written
        self._cfg.flush()
        # Run snmpd
        args = [
            self._path,
            "-C",  # Ignore default configs
            "-c",  # Read our config
            self._cfg.name,
            "-f",  # No fork
            "-Lo",  # Log to stdout
            "-V",  # Verbose
        ]
        if self._log_packets:
            args += [
                "-d",  # Dump packets
            ]
        logger.debug("Running: %s", " ".join(args))
        self._proc = subprocess.Popen(
            args,
            stdout=subprocess.PIPE,
            encoding="utf-8",
            text=True,
        )
        # Wait for snmpd is up
        self._wait()
        self._consume_stdout()

    def _wait_inner(self: "Snmpd", q: "queue.Queue[Optional[str]]") -> None:
        """
        Inner implementation of snmpd waiter.

        Launched from the separate thread.

        Args:
            q: Result queue.
        """
        if self._proc and self._proc.stdout:
            logger.info("Waiting for snmpd")
            for line in self._proc.stdout:
                logger.debug("snmpd: %s", line[:-1])
                if line.startswith("NET-SNMP version"):
                    self.version = line.strip().split(" ", 2)[2].strip()
                    logger.info("snmpd is up. Version %s", self.version)
                    q.put(None)
                    return
            # Premature termination of snmpd
            logging.error("snmpd is terminated prematurely")
            q.put("snmpd is terminated prematurely")
            return
        q.put("snmpd is not active")

    def _wait(self: "Snmpd") -> None:
        """Wait until snmpd is ready."""
        if self._proc is None:
            msg = "_wait() must not be started directly"
            raise RuntimeError(msg)
        if not self._proc.stdout:
            msg = "stdout is not piped"
            raise RuntimeError(msg)
        q: queue.Queue[Optional[str]] = queue.Queue()
        t = threading.Thread(target=self._wait_inner, args=[q])
        t.daemon = True
        t.start()
        try:
            err = q.get(block=True, timeout=self._start_timeout)
        except queue.Empty:
            raise TimeoutError from None
        if err is not None:
            raise RuntimeError(err)
        if t.is_alive():
            logger.error("snmpd is failed to start")
            msg = "snmpd failed to start"
            raise TimeoutError(msg)

    def _consume_stdout(self: "Snmpd") -> None:
        def inner() -> None:
            if self._proc and self._proc.stdout:
                for line in self._proc.stdout:
                    logger.debug("snmpd: %s", line[:-1])

        t = threading.Thread(target=inner)
        t.daemon = True
        t.start()

    def _stop(self: "Snmpd") -> None:
        """Terminate snmpd instance."""
        if self._proc:
            logger.info("Stopping snmpd")
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
