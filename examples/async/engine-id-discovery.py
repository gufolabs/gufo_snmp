import asyncio
import sys

from gufo.snmp import SnmpSession, User


async def main(addr: str, user_name: str) -> None:
    async with SnmpSession(addr=addr, user=User(user_name)) as session:
        engine_id = session.get_engine_id()
        print(engine_id.hex())


asyncio.run(main(sys.argv[1], sys.argv[2]))
