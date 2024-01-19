# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 users
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""User structure definition."""

# Python modules
from dataclasses import dataclass
from typing import List, Optional


class BaseKey(object):
    """Basic key class."""

    KEY_LENGTH: int
    PAD = b"\x00"
    SNMPD_PREFIX: str

    def __init__(self: "BaseKey", key: bytes) -> None:
        kl = len(key)
        self.key: bytes
        if kl > self.KEY_LENGTH:
            self.key = key[: self.KEY_LENGTH]
        elif kl == self.KEY_LENGTH:
            self.key = key
        else:
            self.key = key + self.PAD * (self.KEY_LENGTH - kl)

    def snmpd_key(self: "BaseKey") -> List[str]:
        """Returns key and prefix for createUser."""
        return [self.SNMPD_PREFIX, "-m", self.key.hex()]


class BaseAuthKey(BaseKey):
    """Authentication key base class."""

    AUTH_ALG: int


class Md5Key(BaseAuthKey):
    """MD5 Key."""

    AUTH_ALG = 1
    KEY_LENGTH = 16
    SNMPD_PREFIX = "MD5"


class Sha1Key(BaseAuthKey):
    """SHA-1 Key."""

    AUTH_ALG = 2
    KEY_LENGTH = 20
    SNMPD_PREFIX = "SHA"


class BasePrivKey(BaseKey):
    """Privacy key base class."""

    PRIV_ALG: int


class DesKey(BasePrivKey):
    """Des Key."""

    PRIV_ALG = 1
    KEY_LENGTH = 16
    SNMPD_PREFIX = "DES"


class Aes128Key(BasePrivKey):
    """AES-128 Key."""

    PRIV_ALG = 2
    KEY_LENGTH = 16
    SNMPD_PREFIX = "AES"


@dataclass
class User(object):
    """
    SNMPv3 user.

    Attributes:
        name: user name.
        auth_key: Optional authentication key.
        priv_key: Optional privacy key.
    """

    name: str
    auth_key: Optional[BaseAuthKey] = None
    priv_key: Optional[BasePrivKey] = None

    def require_auth(self: "User") -> bool:
        """
        Chech if user requires authentication.

        Returns:
            True, if user requires authetication
        """
        return self.auth_key is not None

    def get_auth_alg(self: "User") -> int:
        """
        Auth algorithm index.

        Returns:
            * 0 - No auth
            * 1 - MD5
            * 2 - SHA1
        """
        return self.auth_key.AUTH_ALG if self.auth_key else 0

    def get_priv_alg(self: "User") -> int:
        """
        Privacy algorithm index.

        Returns:
            * 0 - No privacy
            * 1 - DES
            * 2 - AES-128
        """
        return self.priv_key.PRIV_ALG if self.priv_key else 0

    def get_auth_key(self: "User") -> bytes:
        """Authentication key."""
        return self.auth_key.key if self.auth_key else b""

    def get_priv_key(self: "User") -> bytes:
        """Privacy key."""
        return self.priv_key.key if self.priv_key else b""

    @property
    def snmpd_rouser(self: "User") -> str:
        """
        `rouser` part of snmpd.conf.

        Returns:
            rouser configuration directive.
        """
        if self.priv_key:
            level = "priv"
        elif self.auth_key:
            level = "auth"
        else:
            level = "noauth"
        return f"rouser {self.name} {level}"

    @property
    def snmpd_create_user(self: "User") -> str:
        """
        createUser part of snmpd.conf.

        Returns:
            createUser configuration directive.
        """
        r = ["createUser", self.name]
        if self.auth_key:
            r += self.auth_key.snmpd_key()
        if self.priv_key:
            r += self.priv_key.snmpd_key()
        return " ".join(r)
