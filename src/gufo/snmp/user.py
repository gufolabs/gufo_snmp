# ---------------------------------------------------------------------
# Gufo SNMP: SNMPv3 users
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""User structure definition."""

# Python modules
from enum import IntEnum
from typing import TypeVar

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

    def snmpd_option(self) -> str:
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
    def is_password(self) -> bool:
        """Check if key type is a password."""
        return self == self.Password

    @property
    def is_master(self) -> bool:
        """Check if key type is a master."""
        return self == self.Master

    @property
    def is_localized(self) -> bool:
        """Check if key type is localized."""
        return self == self.Localized

    @property
    def _is_aligned(self) -> bool:
        """Check if the key type has a fixed length."""
        return self.is_master or self.is_localized

    @property
    def _mask(self) -> int:
        """Returns algorithm mask."""
        return self.value << 6


class KeyExpansion(IntEnum):
    """
    Key expansion policy for AES-192/256 privacy keys.

    Specifies how a localized authentication key is expanded when it is
    shorter than the key required by AES-192 or AES-256. This is relevant
    primarily for MD5 and SHA-1 authentication, whose localized keys are
    shorter than the required AES key.

    Different SNMP implementations use different expansion schemes:

    * ``Blumenthal`` — Blumenthal AES-192/256 key expansion.
    * ``Cisco`` — Cisco/Reeder AES-192/256 key expansion.

    The policy has no effect when the localized key is already long enough
    or when a different privacy protocol is used.

    Attributes:
        Blumenthal: Use the Blumenthal key-expansion scheme.
        Cisco: Use the Cisco/Reeder key-expansion scheme.
    """

    Blumenthal = 1
    Cisco = 2


class BaseKey:
    """
    Basic key class.

    Args:
        key: Key value.
        key_type: Key type.
    """

    AUTH_ALG: int
    SNMPD_PREFIX: str

    def __init__(
        self, key: bytes, /, key_type: KeyType = KeyType.Password
    ) -> None:
        self.key = key
        self.key_type = key_type

    @classmethod
    def get_master_key(cls: type["BaseKey"], passwd: bytes) -> bytes:
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
        cls: type["BaseKey"], master_key: bytes, engine_id: bytes
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

    def _pad(self, key_len: int) -> None:
        """
        Pad key to given length.

        Truncates key if its longer, than desired,
        add trailing zeroes otherwise.

        Args:
            key_len: Desired key length.
        """
        self.key = self._padded(self.key, key_len)

    @classmethod
    def _padded(cls: type["BaseKey"], key: bytes, key_len: int) -> bytes:
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

    def snmpd_key(self) -> list[str]:
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
        self,
        key: bytes,
        /,
        key_type: KeyType = KeyType.Password,
    ) -> None:
        if key_type._is_aligned:
            key = self._padded(key, self.KEY_LENGTH)
        super().__init__(key, key_type)


class Md5Key(BaseAuthKey):
    """MD5 Key."""

    AUTH_ALG = 1  # 2 - Blumenthal, 3 - Cisco
    KEY_LENGTH = 16
    SNMPD_PREFIX = "MD5"


class Sha1Key(BaseAuthKey):
    """SHA-1 Key."""

    AUTH_ALG = 4  #  5 - Blumenthal, 6 - Cisco
    KEY_LENGTH = 20
    SNMPD_PREFIX = "SHA"


class BasePrivKey(BaseKey):
    """Privacy key base class."""

    PRIV_ALG: int
    KEY_LENGTH: int


class DesKey(BasePrivKey):
    """Des Key."""

    PRIV_ALG = 1
    SNMPD_PREFIX = "DES"
    KEY_LENGTH = 16


class Aes128Key(BasePrivKey):
    """AES-128 Key."""

    PRIV_ALG = 2
    SNMPD_PREFIX = "AES"
    KEY_LENGTH = 16


class Aes192Key(BasePrivKey):
    """AES-192 Key."""

    PRIV_ALG = 3
    SNMPD_PREFIX = "AES192"
    KEY_LENGTH = 24


class Aes256Key(BasePrivKey):
    """AES-256 Key."""

    PRIV_ALG = 4
    SNMPD_PREFIX = "AES256"
    KEY_LENGTH = 32


class User:
    """
    SNMPv3 user.

    Args:
        name: user name.
        auth_key: Optional authentication key.
        priv_key: Optional privacy key.
        key_expansion: Key expansion policy for AES-192/256 privacy keys.
    """

    def __init__(
        self,
        name: str,
        *,
        auth_key: BaseAuthKey | None = None,
        priv_key: BasePrivKey | None = None,
        key_expansion: KeyExpansion = KeyExpansion.Blumenthal,
    ) -> None:
        self.name = name
        self.auth_key = auth_key
        self.priv_key = priv_key
        self.key_expansion = key_expansion
        if self.priv_key and not self.auth_key:
            msg = "auth_key must be set to use priv_key"
            raise ValueError(msg)
        if (
            self.priv_key
            and self.auth_key
            and self.priv_key.key_type._is_aligned
        ):
            self.priv_key._pad(self.auth_key.KEY_LENGTH)

    def __str__(self) -> str:
        """str() implementation."""
        return self.name

    def __repr__(self) -> str:
        """repr() implementation."""
        return f"<User {self.name} at {id(self)}>"

    @classmethod
    def default(cls: type["User"]) -> "User":
        """
        Default user without name and keys.

        Returns:
            Default user instance.
        """
        return User(name="")

    def require_auth(self) -> bool:
        """
        Chech if user requires authentication.

        Returns:
            True, if user requires authetication
        """
        return self.auth_key is not None

    def get_auth_alg(self) -> int:
        """Return the authentication algorithm index with the key type mask.

        When the privacy key requires more key material than the
        authentication algorithm provides, the configured key expansion
        policy is encoded into the algorithm index.

        Algorithm indexes:

        * 0 - No auth
        * 1 - MD5
        * 2 - MD5 with Blumenthal key expansion
        * 3 - MD5 with Cisco key expansion
        * 4 - SHA-1
        * 5 - SHA-1 with Blumenthal key expansion
        * 6 - SHA-1 with Cisco key expansion

        The expansion policy only affects algorithms whose key is shorter
        than the selected privacy key.

        Returns:
        Authentication algorithm index with the key type mask applied.
        """
        if not self.auth_key:
            return 0
        alg = self.auth_key.AUTH_ALG
        if (
            self.priv_key
            and self.priv_key.KEY_LENGTH > self.auth_key.KEY_LENGTH
        ):
            alg += self.key_expansion.value
        return alg | self.auth_key.key_type._mask

    def get_priv_alg(self) -> int:
        """
        Privacy algorithm index.

        Algorithms:
            * 0 - No privacy
            * 1 - DES
            * 2 - AES-128
            * 3 - AES-192
            * 4 - AES-256
        KeyType.mask applied
        """
        return (
            self.priv_key.PRIV_ALG | self.priv_key.key_type._mask
            if self.priv_key
            else 0
        )

    def get_auth_key(self) -> bytes:
        """Authentication key."""
        return self.auth_key.key if self.auth_key else b""

    def get_priv_key(self) -> bytes:
        """Privacy key."""
        return self.priv_key.key if self.priv_key else b""

    @property
    def snmpd_rouser(self) -> str:
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
    def snmpd_create_user(self) -> str:
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
            if (
                self.auth_key is not None
                and self.priv_key.KEY_LENGTH > self.auth_key.KEY_LENGTH
                and self.key_expansion == KeyExpansion.Cisco
            ):
                r[-3] = f"{r[-3]}C"
        return " ".join(r)
