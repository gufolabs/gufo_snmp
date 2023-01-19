# ---------------------------------------------------------------------
# Gufo SNMP: Snmpd context manager
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import subprocess
from tempfile import NamedTemporaryFile


class Snmpd(object):
    """
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
        self,
        path: str = "/usr/sbin/snmpd",
        address: str = "127.0.0.1",
        port: int = 10161,
        community: str = "public",
        location: str = "Test",
        contact: str = "test <test@example.com>",
        user: str = "rouser",
    ):
        self._path = path
        self._address = address
        self._port = port
        self._community = community
        self._location = location
        self._contact = contact
        self._user = user
        self._cfg = None
        self._proc = None

    def get_config(self) -> str:
        """
        Generate snmpd config.
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

    def _start(self):
        """
        Run snmpd instance.
        """
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
            ]
        )

    def _stop(self):
        """
        Terminate snmpd instance.
        """
        self._proc.kill()
        self._cfg.close()

    def __enter__(self):
        self._start()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self._stop()

    async def __aenter__(self):
        self._start()

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self._stop()
