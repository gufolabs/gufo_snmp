# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 users
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""User structure definition."""

# Python modules
from enum import IntEnum
from typing import List, Optional, Type, TypeVar

# Gufo SNMP modules
from ._fast import get_localized_key, get_master_key

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

    def snmpd_option(self: "KeyType") -> str:
        """
        Get key option for snmpd.conf.

        Returns:
            Key type prefix like -m, -l, ...
        """
        if self == self.Password:
            return ""
        if self == self.Master:
            return "-m"
        if self == self.Localized:
            return "-l"
        msg = "Unknown key type"
        raise ValueError(msg)

    @property
    def is_password(self: "KeyType") -> bool:
        """Check if key type is a password."""
        return self == self.Password

    @property
    def is_master(self: "KeyType") -> bool:
        """Check if key type is a master."""
        return self == self.Master

    @property
    def is_localized(self: "KeyType") -> bool:
        """Check if key type is localized."""
        return self == self.Localized

    @property
    def _is_aligned(self: "KeyType") -> bool:
        """Check if the key type has a fixed length."""
        return self.is_master or self.is_localized

    @property
    def _mask(self: "KeyType") -> int:
        """Returns algorithm mask."""
        return self.value << 6


class BaseKey(object):
    """
    Basic key class.

    Args:
        key: Key value.
        key_type: Key type.
    """

    AUTH_ALG: int
    SNMPD_PREFIX: str

    def __init__(
        self: "BaseKey", key: bytes, /, key_type: KeyType = KeyType.Password
    ) -> None:
        self.key = key
        self.key_type = key_type

    @classmethod
    def get_master_key(cls: Type["BaseKey"], passwd: bytes) -> bytes:
        """
        Convert password to master key.

        Args:
            passwd: Password

        Returns:
            Master key. Resulting length depends on the algorithm.
        """
        return get_master_key(cls.AUTH_ALG, passwd)

    @classmethod
    def get_localized_key(
        cls: Type["BaseKey"], master_key: bytes, engine_id: bytes
    ) -> bytes:
        """
        Convert master key to localized key.

        Args:
            master_key: Master key, must have size according to algorithm.
            engine_id: SNMP engine id.

        Returns:
            Localized key. Resulting length same as master_key.
        """
        return get_localized_key(cls.AUTH_ALG, master_key, engine_id)

    def _pad(self: "BaseKey", key_len: int) -> None:
        """
        Pad key to given length.

        Truncates key if its longer, than desired,
        add trailing zeroes otherwise.

        Args:
            key_len: Desired key length.
        """
        self.key = self._padded(self.key, key_len)

    @classmethod
    def _padded(cls: Type["BaseKey"], key: bytes, key_len: int) -> bytes:
        """
        Returns string aligned to given length.

        Args:
            key: Key value.
            key_len: Desired key length.

        Returns:
            Aligned and padded key.
        """
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
            v = self.key.decode()
        else:
            v = f"0x{self.key.hex()}"
        return [self.SNMPD_PREFIX, self.key_type.snmpd_option(), v]


class BaseAuthKey(BaseKey):
    """Authentication key base class."""

    AUTH_ALG: int
    KEY_LENGTH: int

    def __init__(
        self: "BaseAuthKey",
        key: bytes,
        /,
        key_type: KeyType = KeyType.Password,
    ) -> None:
        if key_type._is_aligned:
            key = self._padded(key, self.KEY_LENGTH)
        super().__init__(key, key_type)


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
        *,
        auth_key: Optional[BaseAuthKey] = None,
        priv_key: Optional[BasePrivKey] = None,
    ) -> None:
        self.name = name
        self.auth_key = auth_key
        self.priv_key = priv_key
        if self.priv_key and not self.auth_key:
            msg = "auth_key must be set to use priv_key"
            raise ValueError(msg)
        if (
            self.priv_key
            and self.auth_key
            and self.priv_key.key_type._is_aligned
        ):
            self.priv_key._pad(self.auth_key.KEY_LENGTH)

    @classmethod
    def default(cls: Type["User"]) -> "User":
        """
        Default user without name and keys.

        Returns:
            Default user instance.
        """
        return User(name="")

    def require_auth(self: "User") -> bool:
        """
        Chech if user requires authentication.

        Returns:
            True, if user requires authetication
        """
        return self.auth_key is not None

    def get_auth_alg(self: "User") -> int:
        """
        Auth algorithm index with key type mask.

        Algorithms:
            * 0 - No auth
            * 1 - MD5
            * 2 - SHA1

        KeyType.mask applied
        """
        return (
            self.auth_key.AUTH_ALG | self.auth_key.key_type._mask
            if self.auth_key
            else 0
        )

    def get_priv_alg(self: "User") -> int:
        """
        Privacy algorithm index.

        Algorithms:
            * 0 - No privacy
            * 1 - DES
            * 2 - AES-128
        KeyType.mask applied
        """
        return (
            self.priv_key.PRIV_ALG | self.priv_key.key_type._mask
            if self.priv_key
            else 0
        )

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
        CreateUser part of snmpd.conf.

        Returns:
            createUser configuration directive.
        """
        r = ["createUser", self.name]
        if self.auth_key:
            r += self.auth_key.snmpd_key()
        if self.priv_key:
            r += self.priv_key.snmpd_key()
        return " ".join(r)
