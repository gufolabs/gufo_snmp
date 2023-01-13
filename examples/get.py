import sys
import asyncio
from gufo.snmp import SnmpSession


async def main(addr: str, community: str, oid: str) -> None:
    async with SnmpSession(addr=addr, community=community) as session:
        r = await session.get(oid)
        print(r)


asyncio.run(main(sys.argv[1], sys.argv[2], sys.argv[3]))
