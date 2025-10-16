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
from itertools import product
from types import TracebackType
from typing import Any, Iterable, Optional, Tuple, Type

# Gufo SNMP Modules
from gufo.snmp import SnmpVersion
from gufo.snmp.user import (
    Aes128Key,
    BaseAuthKey,
    BasePrivKey,
    DesKey,
    KeyType,
    Md5Key,
    Sha1Key,
    User,
)

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = random.randint(52000, 53999)
SNMPD_PATH = "/usr/sbin/snmpd"
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_SYSTEM_OID = "1.3.6.1.2.1.1"
SNMP_LOCATION_OID = "1.3.6.1.2.1.1.6.0"
SNMP_CONTACT_OID = "1.3.6.1.2.1.1.4.0"


# User name is defined as:
# <user><auth alg><auth key type><priv alg><priv key type>
# Where:
# <auth alg> - authentication algorithm. Matches BaseAuthKey.AUTH_ALG
# * 0 - No auth
# * 1 - MD5
# * 2 - SHA1
# <auth key type> - key type for auth. Matches KeyType
# 0 - Password or not applicabile
# 1 - Master
# 2 - Localized (not used in tests)
# <priv alg> - privacy algorithm. Matches BasePrivKey.KEY_ALG
# 0 - No priv
# 1 - DES
# 2 - AES128
# <priv key type> - key type for priv.
# 0 - Password or not applicabile
# 1 - Master
# 2 - Localized (not used in tests)
# Examples
# * `user0000` - no auth, no priv
# * `user1000` - MD5 auth given as password, no priv
# * `user2120` - SHA1 auth given as master, AES128 given as password.
# Auth key is set as:
# * `<username>pass` - for passwords
# * `<username>key` - for keys
# Priv key is an auth key in uppercase.
# Example:
# `user2121` has auth key `user2121pass` and priv key `USER2121PASS`
def _get_user(name: str) -> User:
    """Generate user from username."""

    def get_key_type(code: str) -> KeyType:
        if code == "0":
            return KeyType.Password
        if code == "1":
            return KeyType.Master
        msg = f"Invalid key type: {code}"
        raise ValueError(msg)

    def get_auth_key(name: str) -> Optional[BaseAuthKey]:
        alg_code = name[4]
        key_type = get_key_type(name[5])
        secret = (
            f"{name}pass" if key_type == KeyType.Password else f"{name}key"
        ).encode()
        if alg_code == "0":
            return None
        if alg_code == "1":
            return Md5Key(secret, key_type=key_type)
        if alg_code == "2":
            return Sha1Key(secret, key_type=key_type)
        msg = f"Invalid auth protocol: {alg_code}"
        raise ValueError(msg)

    def get_priv_key(name: str) -> Optional[BasePrivKey]:
        alg_code = name[6]
        key_type = get_key_type(name[7])
        secret = (
            (f"{name}pass" if key_type == KeyType.Password else f"{name}key")
            .upper()
            .encode()
        )
        if alg_code == "0":
            return None
        if alg_code == "1":
            return DesKey(secret, key_type=key_type)
        if alg_code == "2":
            return Aes128Key(secret, key_type=key_type)
        msg = f"Invalid priv protocol: {alg_code}"
        raise ValueError(msg)

    return User(
        name=name, auth_key=get_auth_key(name), priv_key=get_priv_key(name)
    )


def _iter_users() -> Iterable[User]:
    """Generate all users."""
    key_types = "01"
    for auth_alg, auth_key_type, priv_alg, priv_key_type in product(
        "012", key_types, "012", key_types
    ):
        if auth_alg == "0" and (
            auth_key_type != "0" or priv_alg != "0" or priv_key_type != "0"
        ):
            continue  # All zeroes for no auth
        if priv_alg == "0" and priv_key_type != "0":
            continue  # No key type for no priv
        yield _get_user(
            f"user{auth_alg}{auth_key_type}{priv_alg}{priv_key_type}"
        )


SNMP_USERS = list(_iter_users())

V1 = [{"version": SnmpVersion.v1, "community": SNMP_COMMUNITY}]
V2 = [{"version": SnmpVersion.v2c, "community": SNMP_COMMUNITY}]
V3 = [{"version": SnmpVersion.v3, "user": u} for u in SNMP_USERS]
AUTO_V = [{"community": SNMP_COMMUNITY}, {"user": SNMP_USERS[0]}]
ALL = V1 + V2 + V3 + AUTO_V


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
