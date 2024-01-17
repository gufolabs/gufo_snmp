# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 users
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""User structure definition."""

# Python modules
from dataclasses import dataclass
from typing import Optional


class BaseAuthKey(object):
    """Authentication key base class."""

    KEY_LENGTH: int = 16
    PAD = b"\x00"
    AUTH_ALG: int = 0

    def __init__(self: "BaseAuthKey", key: bytes) -> None:
        kl = len(key)
        self.key: bytes
        if kl > self.KEY_LENGTH:
            self.key = key[: self.KEY_LENGTH]
        elif kl == self.KEY_LENGTH:
            self.key = key
        else:
            self.key = key + self.PAD * (self.KEY_LENGTH - kl)


class Md5Key(BaseAuthKey):
    """MD5 Key."""

    AUTH_ALG = 1


class Sha1Key(BaseAuthKey):
    """SHA-1 Key."""

    AUTH_ALG = 2


@dataclass
class User(object):
    """
    SNMPv3 user.

    Attributes:
        name: user name
    """

    name: str
    auth_key: Optional[BaseAuthKey] = None
    priv_key = None

    def require_auth(self: "User") -> bool:
        """
        Chech if user requires authentication.

        Returns:
            True, if user requires authetication
        """
        return self.auth_key is not None

    def get_auth_alg(self: "User") -> int:
        """
        AuthAlgorithm index.

        Returns:
            * 0 - No auth
            * 1 - MD5
            * 2 - SHA1
        """
        return self.auth_key.AUTH_ALG if self.auth_key else 0

    def get_auth_key(self: "User") -> bytes:
        """Authentication key."""
        return self.auth_key.key if self.auth_key else b""

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
            if isinstance(self.auth_key, Md5Key):
                r += ["MD5", "-m", self.auth_key.key.hex()]
            elif isinstance(self.auth_key, Sha1Key):
                r += ["SHA", "-m", self.auth_key.key.hex()]
        return " ".join(r)
