import sys
import asyncio
from gufo.snmp import SnmpSession


async def main(addr: str, community: str, oid: str) -> None:
    async with SnmpSession(addr=addr, community=community) as session:
        async for k, v in session.getnext("1.3.6"):
            print(f"{k}: {v}")


asyncio.run(main(sys.argv[1], sys.argv[2], sys.argv[3]))
