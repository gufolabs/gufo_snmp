# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 users
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""User structure definition."""

# Python modules
from enum import IntEnum
from typing import List, Optional, TypeVar

K = TypeVar("K", bound="BaseKey")


class KeyType(IntEnum):
    """
    Key type.

    Attributes:
        Password: Raw password (least security).
        Master: Master key (hashed password).
        Localized: Localized key, mixed with engine id.
    """

    Password = 0
    Master = 1
    Localized = 2

    def snmpd_option(self) -> str:
        if self == self.Password:
            return "-a"
        if self == self.Master:
            return "-m"
        if self == self.Localized:
            return "-l"
        msg = "Unknown key type"
        raise ValueError(msg)

    @property
    def is_password(self) -> bool:
        return self == self.Password

    @property
    def is_master(self) -> bool:
        return self == self.Master


class BaseKey(object):
    """Basic key class."""

    ALG_ID: int
    SNMPD_PREFIX: str

    def __init__(
        self: "BaseKey", key: bytes, key_type: KeyType = KeyType.Master
    ) -> None:
        self.key = key
        self.key_type = key_type

    def pad(self, key_len: int) -> None:
        self.key = self.padded(self.key, key_len)

    @classmethod
    def padded(cls, key: bytes, key_len: int) -> bytes:
        kl = len(key)
        if kl == key_len:
            return key
        if kl > key_len:
            # Truncate
            return key[:key_len]
        return key + b"\x00" * (key_len - kl)

    def snmpd_key(self: "BaseKey") -> List[str]:
        """Returns key and prefix for createUser."""
        if self.key_type.is_password:
            v = f'"{self.key}"'
        else:
            v = f"0x{self.key.hex()}"
        return [self.SNMPD_PREFIX, self.key_type.snmpd_option(), v]


class BaseAuthKey(BaseKey):
    """Authentication key base class."""

    AUTH_ALG: int
    KEY_LENGTH: int

    def __init__(
        self: "BaseAuthKey", key: bytes, key_type: KeyType = KeyType.Master
    ) -> None:
        if key_type.is_master:
            key = self.padded(key, self.KEY_LENGTH)
        super().__init__(key, key_type)

    @classmethod
    def to_master(cls, key: K) -> K:
        if not key.key_type.is_password:
            msg = "Password key type required"
            raise ValueError(msg)
        raise NotImplementedError

    @classmethod
    def to_localized(cls, key: K, engine_id: bytes) -> K:
        if key.key_type.is_password:
            # Convert password to master key
            key = cls.to_master(key)
        if not key.key_type.is_master:
            msg = "Master key type required"
            raise ValueError(msg)
        raise NotImplementedError


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
    SNMPD_PREFIX = "DES"


class Aes128Key(BasePrivKey):
    """AES-128 Key."""

    PRIV_ALG = 2
    SNMPD_PREFIX = "AES"


class User(object):
    """
    SNMPv3 user.

    Args:
        name: user name.
        auth_key: Optional authentication key.
        priv_key: Optional privacy key.
    """

    def __init__(
        self: "User",
        name: str,
        auth_key: Optional[BaseAuthKey] = None,
        priv_key: Optional[BasePrivKey] = None,
    ):
        self.name = name
        self.auth_key = auth_key
        self.priv_key = priv_key
        if self.priv_key and not self.auth_key:
            msg = "auth_key must be set to use priv_key"
            raise ValueError(msg)
        if (
            self.priv_key
            and self.auth_key
            and self.priv_key.key_type.is_master
        ):
            self.priv_key.pad(self.auth_key.KEY_LENGTH)

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
