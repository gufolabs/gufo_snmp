import asyncio

from gufo.snmp import SnmpSession
from gufo.snmp.snmpd import Snmpd


async def main() -> None:
    async with Snmpd(), SnmpSession(addr="127.0.0.1", port=10161) as session:
        async for oid, value in session.getnext("1.3.6.1.2.1.1"):
            print(f"{oid}: {value}")


asyncio.run(main())
