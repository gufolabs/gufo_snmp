# ---------------------------------------------------------------------
# Gufo SNMP: User definitions and test utilities.
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import random
import socket
import threading
import time
from contextlib import suppress
from types import TracebackType
from typing import Any, Optional, Tuple, Type

# Gufo SNMP Modules
from gufo.snmp import SnmpVersion
from gufo.snmp.user import Aes128Key, DesKey, KeyType, Md5Key, Sha1Key, User

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = random.randint(52000, 53999)
SNMPD_PATH = "/usr/sbin/snmpd"
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_USERS = [
    User(name="user00"),
    # MD5
    User(
        name="user10", auth_key=Md5Key(b"user10key", key_type=KeyType.Master)
    ),
    User(
        name="user10p",
        auth_key=Md5Key(b"user10pass", key_type=KeyType.Password),
    ),
    User(
        name="user11",
        auth_key=Md5Key(b"user11key", key_type=KeyType.Master),
        priv_key=DesKey(b"USER11KEY", key_type=KeyType.Master),
    ),
    User(
        name="user11p",
        auth_key=Md5Key(b"user11pass", key_type=KeyType.Password),
        priv_key=DesKey(b"USER11PASS", key_type=KeyType.Password),
    ),
    User(
        name="user12",
        auth_key=Md5Key(b"user11key", key_type=KeyType.Master),
        priv_key=Aes128Key(b"USER12KEY", key_type=KeyType.Master),
    ),
    # SHA1
    User(
        name="user20", auth_key=Sha1Key(b"user20key", key_type=KeyType.Master)
    ),
    User(
        name="user21",
        auth_key=Sha1Key(b"user21key", key_type=KeyType.Master),
        priv_key=DesKey(b"USER21KEY", key_type=KeyType.Master),
    ),
    User(
        name="user22",
        auth_key=Sha1Key(b"user22key", key_type=KeyType.Master),
        priv_key=Aes128Key(b"USER22KEY", key_type=KeyType.Master),
    ),
]

V1 = [{"version": SnmpVersion.v1, "community": SNMP_COMMUNITY}]
V2 = [{"version": SnmpVersion.v2c, "community": SNMP_COMMUNITY}]
V3 = [{"version": SnmpVersion.v3, "user": u} for u in SNMP_USERS]

ALL = V1 + V2 + V3

SNMP_LOCATION_OID = "1.3.6.1.2.1.1.6.0"
SNMP_CONTACT_OID = "1.3.6.1.2.1.1.4.0"


def ids(x: Any) -> str:
    if isinstance(x, dict) and "version" in x:
        r = [x["version"].name]
        user = x.get("user")
        if user:
            r += [user.name]
            if user.auth_key:
                r += [user.auth_key.__class__.__name__]
            if user.priv_key:
                r += [user.priv_key.__class__.__name__]
        return "-".join(r)
    return str(x)


class SyncShiftProxy(object):
    """
    A shifting proxy, sync version.

    Drops first reply, then returns
    a previous reply and then actual one.
    """

    def __init__(self: "SyncShiftProxy") -> None:
        self._listen_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._listen_sock.bind(("127.0.0.1", 0))
        self._addr: Tuple[str, int] = self._listen_sock.getsockname()
        self._proxy_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._thread: Optional[threading.Thread] = None

    def __enter__(self: "SyncShiftProxy") -> "SyncShiftProxy":
        """Context management entry."""
        self._thread = threading.Thread(target=self.run, name="ShiftProxyy")
        self._thread.daemon = True
        self._thread.start()
        return self

    def __exit__(
        self: "SyncShiftProxy",
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        """Context management exit."""
        print(exc_type, time.time())
        self.close()
        if self._thread:
            self._thread.join(1.0)
            self._thread = None

    @property
    def addr(self: "SyncShiftProxy") -> Tuple[str, int]:
        """
        Get address info.

        Returns:
            Tuple of addr, port
        """
        return self._addr

    def run(self: "SyncShiftProxy") -> None:
        """Run proxy."""
        with suppress(OSError):
            self._run()

    def _run(self: "SyncShiftProxy") -> None:
        """Run proxy (internal implementation)."""
        BUFF_SIZE = 4096
        # Receive request
        r, addr = self._listen_sock.recvfrom(BUFF_SIZE)
        print("CLIENT -> PROXY    SERVER")
        # Proxy it
        self._proxy_sock.sendto(r, (SNMPD_ADDRESS, SNMPD_PORT))
        print("CLIENT    PROXY -> SERVER")
        # Get reply from server
        delayed, _ = self._proxy_sock.recvfrom(BUFF_SIZE)
        print("CLIENT    PROXY <- SERVER")
        # Do not send delayed reply, wait for next request
        r, addr = self._listen_sock.recvfrom(BUFF_SIZE)
        print("CLIENT -> PROXY    SERVER")
        # Proxy next request
        self._proxy_sock.sendto(r, (SNMPD_ADDRESS, SNMPD_PORT))
        print("CLIENT    PROXY -> SERVER")
        # Get reply from server
        reply, _ = self._proxy_sock.recvfrom(BUFF_SIZE)
        print("CLIENT    PROXY <- SERVER")
        # Send delayed reply for 3 times
        for _ in range(3):
            self._listen_sock.sendto(delayed, addr)
            print("CLIENT <- PROXY    SERVER")
            # Wait for while
            time.sleep(0.1)
        # Send real reply
        self._listen_sock.sendto(reply, addr)
        print("CLIENT <- PROXY    SERVER")

    def close(self: "SyncShiftProxy") -> None:
        """Close sockets."""
        self._listen_sock.close()
        self._proxy_sock.close()
