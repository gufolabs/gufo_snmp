# ---------------------------------------------------------------------
# Gufo SNMP: User definitions and test utilities.
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import random
import socket
import sys
import threading
import time
from collections.abc import Iterable
from contextlib import suppress
from itertools import product
from types import TracebackType
from typing import Any

# Gufo SNMP Modules
from gufo.snmp import SnmpVersion
from gufo.snmp.user import (
    Aes128Key,
    Aes192Key,
    Aes256Key,
    BaseAuthKey,
    BasePrivKey,
    DesKey,
    KeyExpansion,
    KeyType,
    Md5Key,
    Sha1Key,
    Sha224Key,
    Sha256Key,
    Sha384Key,
    Sha512Key,
    User,
)

SNMPD_ADDRESS = "127.0.0.1"
SNMPD_PORT = random.randint(52000, 53999)
SNMP_COMMUNITY = "public"
SNMP_LOCATION = "Gufo SNMP Test"
SNMP_CONTACT = "test <test@example.com>"
SNMP_SYSTEM_OID = "1.3.6.1.2.1.1"
SNMP_LOCATION_OID = "1.3.6.1.2.1.1.6.0"
SNMP_CONTACT_OID = "1.3.6.1.2.1.1.4.0"


def _get_key_type(code: str) -> KeyType:
    match code:
        case "0":
            return KeyType.Password
        case "1":
            return KeyType.Master
        case _:
            msg = f"Invalid key type: {code}"
            raise ValueError(msg)


def _get_auth_key(name: str) -> BaseAuthKey | None:  # noqa: PLR0911
    alg_code = name[4]
    key_type = _get_key_type(name[5])
    secret = (
        f"{name}pass" if key_type == KeyType.Password else f"{name}key"
    ).encode()
    match alg_code:
        case "0":
            return None
        case "1":
            return Md5Key(secret, key_type=key_type)
        case "2":
            return Sha1Key(secret, key_type=key_type)
        case "3":
            return Sha224Key(secret, key_type=key_type)
        case "4":
            return Sha256Key(secret, key_type=key_type)
        case "5":
            return Sha384Key(secret, key_type=key_type)
        case "6":
            return Sha512Key(secret, key_type=key_type)
        case _:
            msg = f"Invalid auth protocol: {alg_code}"
            raise ValueError(msg)


def _get_priv_key(name: str) -> BasePrivKey | None:
    alg_code = name[6]
    key_type = _get_key_type(name[7])
    secret = (
        (f"{name}pass" if key_type == KeyType.Password else f"{name}key")
        .upper()
        .encode()
    )
    match alg_code:
        case "0":
            return None
        case "1":
            return DesKey(secret, key_type=key_type)
        case "2":
            return Aes128Key(secret, key_type=key_type)
        case "3" | "4":
            return Aes192Key(secret, key_type=key_type)
        case "5" | "6":
            return Aes256Key(secret, key_type=key_type)
        case _:
            msg = f"Invalid priv protocol: {alg_code}"
            raise ValueError(msg)


def _get_key_expansion(name: str) -> KeyExpansion:
    alg_code = name[6]
    if alg_code in {"4", "6"}:
        return KeyExpansion.Cisco
    return KeyExpansion.Blumenthal


def _get_user(name: str) -> User:
    """Generate user from username.

    User name is defined as:

    ```
    <user><auth alg><auth key type><priv alg><priv key type>
    ```

    Where:
    - `<auth alg>` - authentication algorithm. Matches BaseAuthKey.AUTH_ALG

        * `0` - No auth
        * `1` - MD5
        * `2` - SHA1
        * `3` - SHA-224
        * `4` - SHA-256
        * `5` - SHA-384
        * `6` - SHA-512

    - `<auth key type>` - key type for auth. Matches KeyType

        * `0` - Password or not applicabile
        * `1` - Master
        * `2` - Localized (not used in tests)

    - `<priv alg>` - privacy algorithm. Matches BasePrivKey.KEY_ALG

        `0` - No priv
        `1` - DES
        `2` - AES128
        `3` - AES192 + Blumethal
        `4` - AES192 + Cisco
        `5` - AES256 + Blumenthal
        `6` - AES256 + Cisco

    - `<priv key type>` - key type for priv.

        `0` - Password or not applicabile
        `1` - Master
        `2` - Localized (not used in tests)

    Examples:
        * `user0000` - no auth, no priv
        * `user1000` - MD5 auth given as password, no priv
        * `user2120` - SHA1 auth given as master, AES128 given as password.

    Auth key is set as:

    * `<username>pass` - for passwords
    * `<username>key` - for keys
    Priv key is an auth key in uppercase.

    Example:
        `user2121` has auth key `user2121pass` and priv key `USER2121PASS`
    """
    return User(
        name=name,
        auth_key=_get_auth_key(name),
        priv_key=_get_priv_key(name),
        key_expansion=_get_key_expansion(name),
    )


if sys.platform == "darwin":

    def _is_allowed_user(username: str) -> bool:
        auth_alg = username[-4]
        if auth_alg in {
            "3",  # SHA-224
            "4",  # SHA-256
            "5",  # SHA-384
            "6",  # SHA-512
        }:
            return False
        priv_alg = username[-2]
        return priv_alg not in {
            "3",  # AES192 + Blumethal
            "4",  # AES192 + Cisco
            "5",  # AES256 + Blumenthal
            "6",  # AES256 + Cisco
        }
else:

    def _is_allowed_user(username: str) -> bool:
        return True


def _iter_users() -> Iterable[User]:
    """Generate all users."""
    key_types = "01"
    for auth_alg, auth_key_type, priv_alg, priv_key_type in product(
        "0123456", key_types, "0123456", key_types
    ):
        if auth_alg == "0" and (
            auth_key_type != "0" or priv_alg != "0" or priv_key_type != "0"
        ):
            continue  # All zeroes for no auth
        if priv_alg == "0" and priv_key_type != "0":
            continue  # No key type for no priv
        user_name = f"user{auth_alg}{auth_key_type}{priv_alg}{priv_key_type}"
        if not _is_allowed_user(user_name):
            continue
        yield _get_user(user_name)


SNMP_USERS = list(_iter_users())

V1 = [{"version": SnmpVersion.v1, "community": SNMP_COMMUNITY}]
V2 = [{"version": SnmpVersion.v2c, "community": SNMP_COMMUNITY}]
V3 = [{"version": SnmpVersion.v3, "user": u} for u in SNMP_USERS]
AUTO_V = [{"community": SNMP_COMMUNITY}, {"user": SNMP_USERS[0]}]
ALL = V1 + V2 + V3 + AUTO_V
UNAUTH_V3_USER = User(name="user2121")


def ids(x: Any) -> str:
    if isinstance(x, dict) and "version" in x:
        r = [x["version"].name]
        user: User | None = x.get("user")
        if user:
            r.append(user.name)
            if user.auth_key:
                r.append(user.auth_key.__class__.__name__)
            if user.priv_key:
                r.append(user.priv_key.__class__.__name__)
                if (
                    user.auth_key
                    and user.priv_key.KEY_LENGTH > user.auth_key.KEY_LENGTH
                ):
                    match user.key_expansion:
                        case KeyExpansion.Blumenthal:
                            r.append("BLU")
                        case KeyExpansion.Cisco:
                            r.append("CIS")
                        case _:
                            pass
        return "-".join(r)
    return str(x)


class SyncShiftProxy:
    """
    A shifting proxy, sync version.

    Drops first reply, then returns
    a previous reply and then actual one.
    """

    def __init__(self) -> None:
        self._listen_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._listen_sock.bind(("127.0.0.1", 0))
        self._addr: tuple[str, int] = self._listen_sock.getsockname()
        self._proxy_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._thread: threading.Thread | None = None

    def __enter__(self) -> "SyncShiftProxy":
        """Context management entry."""
        self._thread = threading.Thread(target=self.run, name="ShiftProxyy")
        self._thread.daemon = True
        self._thread.start()
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        """Context management exit."""
        print(exc_type, time.time())
        self.close()
        if self._thread:
            self._thread.join(1.0)
            self._thread = None

    @property
    def addr(self) -> tuple[str, int]:
        """
        Get address info.

        Returns:
            Tuple of addr, port
        """
        return self._addr

    def run(self) -> None:
        """Run proxy."""
        with suppress(OSError):
            self._run()

    def _run(self) -> None:
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

    def close(self) -> None:
        """Close sockets."""
        self._listen_sock.close()
        self._proxy_sock.close()
