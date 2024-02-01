from gufo.snmp.snmpd import Snmpd
from gufo.snmp.sync_client import SnmpSession


def main() -> None:
    with Snmpd(), SnmpSession(addr="127.0.0.1", port=10161) as session:
        for oid, value in session.getnext("1.3.6.1.2.1.1"):
            print(f"{oid}: {value}")


main()
