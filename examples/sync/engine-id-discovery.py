import sys

from gufo.snmp import User
from gufo.snmp.sync_client import SnmpSession


def main(addr: str, user_name: str) -> None:
    with SnmpSession(addr=addr, user=User(user_name)) as session:
        engine_id = session.get_engine_id()
        print(engine_id.hex())


main(sys.argv[1], sys.argv[2])
