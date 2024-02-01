import sys

from gufo.snmp import Aes128Key, DesKey, Md5Key, Sha1Key, User
from gufo.snmp.sync_client import SnmpSession

AUTH_ALG = {
    "md5": Md5Key,
    "sha": Sha1Key,
}

PRIV_ALG = {
    "des": DesKey,
    "aes128": Aes128Key,
}


def get_user() -> User:
    name = sys.argv[2]
    if len(sys.argv) > 4:
        auth_alg, key = sys.argv[4].split(":", 1)
        auth_key = AUTH_ALG[auth_alg](key.encode())
    else:
        auth_key = None
    if len(sys.argv) > 5:
        priv_alg, key = sys.argv[5].split(":", 1)
        priv_key = PRIV_ALG[priv_alg](key.encode())
    else:
        priv_key = None
    return User(name, auth_key=auth_key, priv_key=priv_key)


def main(addr: str, user: User, oid: str) -> None:
    with SnmpSession(addr=addr, user=user) as session:
        r = session.get(oid)
        print(r)


main(sys.argv[1], get_user(), sys.argv[3])
