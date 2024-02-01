import sys

from gufo.snmp.sync_client import SnmpSession


def main(addr: str, community: str, oid: str) -> None:
    with SnmpSession(addr=addr, community=community) as session:
        r = session.get(oid)
        print(r)


main(sys.argv[1], sys.argv[2], sys.argv[3])
