# Gufo SNMP

*The accelerated Python SNMP client library.*

[![PyPi version](https://img.shields.io/pypi/v/gufo_snmp.svg)](https://pypi.python.org/pypi/gufo_snmp/)
![Downloads](https://img.shields.io/pypi/dw/gufo_snmp)
![Python Versions](https://img.shields.io/pypi/pyversions/gufo_snmp)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
![Build](https://img.shields.io/github/actions/workflow/status/gufolabs/gufo_snmp/tests.yml?branch=master)
[![codecov](https://codecov.io/gh/gufolabs/gufo_snmp/graph/badge.svg?token=QZ2OBDX4H2)](https://codecov.io/gh/gufolabs/gufo_snmp)
![Sponsors](https://img.shields.io/github/sponsors/gufolabs)
[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/charliermarsh/ruff/main/assets/badge/v0.json)](https://github.com/charliermarsh/ruff)

---

**Documentation**: [https://docs.gufolabs.com/gufo_snmp/](https://docs.gufolabs.com/gufo_snmp/)

**Source Code**: [https://github.com/gufolabs/gufo_snmp/](https://github.com/gufolabs/gufo_snmp/)

---

*Gufo SNMP* is the accelerated Python SNMP client library supporting both async and synchronous mode.
It consists of a clean Python API for high-efficient BER parser
and socket IO, implemented in the 
[Rust][Rust] language with [PyO3][PyO3] wrapper.

The querying of the single MIB key is a simple task:

``` py
from gufo.snmp import SnmpSession

async with SnmpSession(addr="127.0.0.1", community="public") as session:
    r = await session.get("1.3.6.1.2.1.1.3.0")
```

And the blocking mode shares virtually the same API:

``` py
from gufo.snmp.sync_client import SnmpSession

with SnmpSession(addr="127.0.0.1", community="public") as session:
    r = session.get("1.3.6.1.2.1.1.3.0")
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

SNMPv3 shares same API and semantics:

``` py
async with SnmpSession(
    addr="127.0.0.1",
    user=User(
        "user1",
        auth_key=Sha1Key(b"12345678"),
        priv_key=Aes128Key(b"87654321")
    )
) as session:
    r = await session.get("1.3.6.1.2.1.1.3.0")
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

## Features

* Clean async and blocking API.
* SNMP v1/v2c/v3 support.
* SNMP v3 User Security Model:
    * Authentication: HMAC-MD5-96, HMAC-SHA-96.
    * Privacy: DES, AES128.
    * Engine ID discovery.
* High-performance.
* Built with security in mind.
* Zero-copy BER parsing.
* Query rate limiting.
* Full Python typing support.
* Editor completion.
* Well-tested, battle-proven code.
* Thoroughly check compatibility with various network equipment.

## Further Roadmap

* SHA2 family of hashes.
* AES256 encryption.
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
[Gufo Stack]: https://docs.gufolabs.com/
[NOC]: https://getnoc.com/
[Rust]: https://rust-lang.org/
[PyO3]: https://pyo3.rs/