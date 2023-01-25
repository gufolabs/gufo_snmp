import asyncio
import sys
from typing import List

from gufo.snmp import SnmpSession


async def main(addr: str, community: str, oids: List[str]) -> None:
    async with SnmpSession(addr=addr, community=community) as session:
        r = await session.get_many(oids)
        for k, v in r.items():
            print(f"{k}: {v}")


asyncio.run(main(sys.argv[1], sys.argv[2], list(sys.argv[3:])))
