# Gufo SNMP Documentation

*Gufo SNMP* is the accelerated Python asyncio SNMP client library.
It consists of a clean Python API for high-efficient BER parser
and socket IO, implemented in the 
[Rust][Rust] language with [PyO3][PyO3] wrapper.

The querying of the single MIB key is a simple task:

``` py
async with SnmpSession(addr="127.0.0.1", community="public") as session:
    r = await session.get("1.3.6.1.2.1.1.3.0")
```

Multiple keys can be queried by one request too:

``` py
async with SnmpSession(addr="127.0.0.1", community="public") as session:
    r = await session.get_many(["1.3.6.1.2.1.1.3.0", "1.3.6.1.2.1.1.2.0"])
```

The querying of the MIB parts is also available with GetNext request:

``` py
async with SnmpSession(addr="127.0.0.1", community="public") as session:
    async for oid, value in  session.getnext("1.3.6.1.2.1.1"):
        ...
```

And with GetBulk request:

``` py
async with SnmpSession(addr="127.0.0.1", community="public") as session:
    async for oid, value in  session.getbulk("1.3.6.1.2.1.1"):
        ...
```

The `.fetch()` method allows to choose between `.getnext()` and `.getbulk()` automatically:
``` py
async with SnmpSession(addr="127.0.0.1", community="public") as session:
    async for oid, value in  session.fetch("1.3.6.1.2.1.1"):
        ...
```

*Gufo SNMP* also allows to limit rate of outgoing requests to protect equipment
from overloading:

``` py
async with SnmpSession(addr="127.0.0.1", community="public", limit_rps=10) as session:
    async for oid, value in  session.fetch("1.3.6.1.2.1.1"):
        ...
```

*Gufo SNMP* offers various tools for developers, including a wrapper to
run a local instance of SNMP daemon:

``` py
async with Snmpd(), SnmpSession(addr="127.0.0.1", port=10161) as session:
    r = await session.get("1.3.6.1.2.1.1.3.0")
```

## Virtues

* Clean async API.
* SNMP v1/v2c support.
* High-performance.
* Zero-copy BER parsing.
* Full Python typing support.
* Query rate limiting.
* Editor completion.
* Well-tested, battle-proven code.

## Further Roadmap

* SNMPv3 support.
* SNMP Trap and Inform collector.
* Incorporation of the [NOC's][NOC] *Compiled MIB* infrastructure.

## On Gufo Stack

This product is a part of [Gufo Stack][Gufo Stack] - the collaborative effort 
led by [Gufo Labs][Gufo Labs]. Our goal is to create a robust and flexible 
set of tools to create network management software and automate 
routine administration tasks.

To do this, we extract the key technologies that have proven themselves 
in the [NOC][NOC] and bring them as separate packages. Then we work on API,
performance tuning, documentation, and testing. The [NOC][NOC] uses the final result
as the external dependencies.

[Gufo Stack][Gufo Stack] makes the [NOC][NOC] better, and this is our primary task. But other products
can benefit from [Gufo Stack][Gufo Stack] too. So we believe that our effort will make 
the other network management products better.

[Gufo Labs]: https://gufolabs.com/
[Gufo Stack]: https://gufolabs.com/products/gufo-stack/
[NOC]: https://getnoc.com/
[Rust]: https://rust-lang.org/
[PyO3]: https://pyo3.rs/