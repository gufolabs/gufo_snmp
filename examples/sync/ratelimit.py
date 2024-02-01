import sys

from gufo.snmp.sync_client import SnmpSession


def main(addr: str, community: str, oid: str) -> None:
    with SnmpSession(
        addr=addr, community=community, allow_bulk=True, limit_rps=10
    ) as session:
        for k, v in session.fetch(oid):
            print(f"{k}: {v}")


main(sys.argv[1], sys.argv[2], sys.argv[3])
