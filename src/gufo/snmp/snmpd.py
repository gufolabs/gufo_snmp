# ---------------------------------------------------------------------
# Gufo SNMP: Snmpd context manager
# ---------------------------------------------------------------------
# Copyright (C) 2023-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""snmpd context manager."""

# Python modules
import logging
import os
import queue
import random
import shutil
import string
import subprocess
import sys
import threading
from tempfile import (
    NamedTemporaryFile,
    TemporaryDirectory,
    _TemporaryFileWrapper,
)
from types import TracebackType
from typing import List, Optional, Type

# Gufo SNMP modules
from .user import User

logger = logging.getLogger("gufo.snmp.snmpd")

# Net-snmp always adds prefix to engine id
_NETSNMP_ENGINE_ID_PREFIX = b"\x80\x00\x1f\x88\x04"
# Length of generated engine id
# Not including prefix.
_ENGINE_ID_LENGTH = 8

IS_DARWIN = sys.platform == "darwin"


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
        engine_id: Optional explicit engine id for SNMPv3.
            Use generated value if not set.
        users: Optional list of SNMPv3 users.
        start_timeout: Maximum time to wait for snmpd to start.
        verbose: Verbose output.
        log_packets: Log SNMP requests and responses,
            available only with `verbose` option.

    Attributes:
        version: Net-SNMP version.
        engine_id: SNMPv3 engine id.

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
        path: Optional[str] = None,
        address: str = "127.0.0.1",
        port: int = 10161,
        community: str = "public",
        location: str = "Test",
        contact: str = "test <test@example.com>",
        engine_id: Optional[str] = None,
        users: Optional[List[User]] = None,
        start_timeout: float = 5.0,
        verbose: bool = False,
        log_packets: bool = False,
    ) -> None:
        self._path = path or self._get_snmpd_path()
        self._address = address
        self._port = port
        self._community = community
        self._location = location
        self._contact = contact
        self._users = users or [User(name="rouser")]
        self._start_timeout = start_timeout
        self._verbose = verbose
        self._log_packets = log_packets if verbose else False
        self.version: Optional[str] = None
        self._cfg: Optional[_TemporaryFileWrapper[str]] = None
        self._persistent_dir: Optional[TemporaryDirectory[str]] = None
        self._proc: Optional[subprocess.Popen[str]] = None
        if engine_id:
            self._cfg_engine_id = engine_id
        else:
            self._cfg_engine_id = self._get_engine_id()
        self.engine_id = (
            _NETSNMP_ENGINE_ID_PREFIX + self._cfg_engine_id.encode()
        )

    @staticmethod
    def _get_engine_id() -> str:
        """
        Generate random engine id.

        Returns:
            Random engine id for snmpd.conf
        """
        chars = string.ascii_letters + string.digits
        return "".join(random.choices(chars, k=_ENGINE_ID_LENGTH))  # noqa:S311

    def get_config(self: "Snmpd") -> str:
        """
        Generate snmpd config.

        Returns:
            snmpd configuration.
        """
        rousers = "\n".join(u.snmpd_rouser for u in self._users)
        create_users = "\n".join(u.snmpd_create_user for u in self._users)
        return f"""# Gufo SNMP Test Suite
master agentx
agentaddress udp:{self._address}:{self._port}
agentXsocket tcp:{self._address}:{self._port}
# SNMPv3 engine id
engineId {self._cfg_engine_id}
# Listen address
# SNMPv1/SNMPv2c R/O community
rocommunity {self._community} 127.0.0.1
# SNMPv3 R/O User
{rousers}
{create_users}
# System information
syslocation {self._location}
syscontact  {self._contact}
#
sysServices 72"""

    # @todo: createUser
    # http://www.net-snmp.org/docs/man/snmpd.conf.html

    def _start(self: "Snmpd") -> None:
        """Run snmpd instance."""
        logger.info("Starting snmpd instance")
        self._cfg = NamedTemporaryFile(  # noqa: SIM115
            prefix="snmpd-", suffix=".conf", mode="w"
        )
        self._persistent_dir = TemporaryDirectory()
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
        ]
        if IS_DARWIN:
            # HOST-RESOURCES-MIB::swRunPath is too large on MacOS
            # causing packet sending arror and test timeouts.
            args += ["-I", "-hrSWRunTable,hrSWInstalledTable"]
        if self._verbose:
            args += ["-V"]
        if self._log_packets:
            args += ["-d"]  # Dump packets
            args += ["-Ddump,usm"]
        logger.debug("Running: %s", " ".join(args))
        self._proc = subprocess.Popen(
            args,
            stdout=subprocess.PIPE,
            encoding="utf-8",
            text=True,
            env={"SNMP_PERSISTENT_DIR": self._persistent_dir.name},
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
            self._cfg = None
        if self._persistent_dir:
            self._persistent_dir.cleanup()
            self._persistent_dir = None

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

    @property
    def address(self) -> str:
        """Get listen address."""
        return self._address

    @property
    def port(self) -> int:
        """Get listen port."""
        return self._port

    @staticmethod
    def _get_snmpd_path() -> str:
        """Detect snmpd's path."""
        # Default place
        path = "/usr/sbin/snmpd"
        # Darwin and others
        if not os.path.exists(path):
            path = shutil.which("snmpd") or ""
        return path
