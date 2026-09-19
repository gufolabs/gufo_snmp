import sys

from gufo.snmp.sync_client import SnmpSession


def main(addr: str, community: str, oids: list[str]) -> None:
    with SnmpSession(addr=addr, community=community) as session:
        r = session.get_many(oids)
        for k, v in r.items():
            print(f"{k}: {v}")


main(sys.argv[1], sys.argv[2], list(sys.argv[3:]))
