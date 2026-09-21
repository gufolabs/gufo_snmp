---
hide:
    - navigation
---
# FAQ

## Getting Started

### What is Gufo SNMP?

Gufo SNMP is a modern, high-performance SNMP library for Python. It provides a simple Python API for working with SNMP devices while using Rust for performance-critical operations.

### Why should I use Gufo SNMP?

Gufo SNMP combines the simplicity of Python with the performance of Rust. It is a good fit for applications that need to communicate with many SNMP devices efficiently without giving up a convenient Python API.

### Who is Gufo SNMP for?

Gufo SNMP is for network engineers, system administrators, and Python developers building network monitoring, management, automation, and observability applications. It is particularly useful for applications that need to communicate with many SNMP devices efficiently.

### Is Gufo SNMP a replacement for Net-SNMP?

While Net-SNMP is a comprehensive framework covering virtually every aspect of SNMP, Gufo SNMP takes a different approach. It hides the complexity of the protocol from the developer behind a simple, clean API, while delivering high performance through its Rust-powered core.

### Does Gufo SNMP work with existing SNMP devices?

Yes. Gufo SNMP works with existing SNMP-enabled devices. If a device supports SNMP, Gufo SNMP can communicate with it.

### Which versions of SNMP are supported?

Gufo SNMP supports SNMPv1, SNMPv2c, and SNMPv3.

### How do I install Gufo SNMP?

Gufo SNMP can be installed using standard Python package managers such as pip or uv

```shell
pip install gufo-snmp
```

```shell
uv add gufo-snmp
```

### Do you provide pre-built binary wheels?

Yes. Gufo SNMP provides pre-built binary wheels for supported platforms. You don't need to install Rust or compile anything from source for a regular installation.

### How do I perform my first SNMP request?

Create an SnmpSession with the device address and community, then use `get()` to retrieve an OID:

```python
from gufo.snmp.sync import SnmpSession

with SnmpSession(addr="192.0.2.1", community="public") as session:
    print(session.get("1.3.6.1.2.1.1.1.0"))
```

## SNMP Features

### Does Gufo SNMP support SNMPv3?

Yes. Gufo SNMP provides support for SNMPv3, including authentication and privacy (encryption). Supported authentication and privacy protocols are described below.

### Which SNMPv3 authentication methods are supported?

Gufo SNMP supports the following SNMPv3 authentication methods:

* **MD5**
* **SHA-1**
* **SHA-224**
* **SHA-256**
* **SHA-384**
* **SHA-512**

### Which SNMPv3 privacy protocols are supported?

Gufo SNMP supports the following SNMPv3 privacy protocols:

* **DES**
* **AES-128**
* **AES-192** (Blumenthal and Cisco/Reeder)
* **AES-256** (Blumenthal and Cisco/Reeder)

### Does Gufo SNMP support GET, GETNEXT, and GETBULK?

Yes. Gufo SNMP supports the standard SNMP GET, GETNEXT, and GETBULK operations.

### Can I walk an SNMP tree?

Yes. Gufo SNMP supports walking an SNMP tree using GETNEXT or GETBULK. The high-level API handles the iteration for you:

```python
for oid, value in session.fetch("1.3.6.1.2.1"):
    print(oid, value)
```

### Does Gufo SNMP support SNMP traps?

Not yet. SNMP trap support is planned for a future release.

### Does Gufo SNMP support INFORMs?

Not yet. SNMP INFORM support is planned for a future release.

### Does Gufo SNMP support IPv6?

Yes. Gufo SNMP supports SNMP over IPv6.

### Can I use UDP and TCP?

Gufo SNMP currently supports SNMP over UDP. TCP support is not currently available.

### Can I query multiple OIDs in one request?

Yes. Gufo SNMP supports querying multiple OIDs in a single request using `get_many()`

### Does Gufo SNMP support SNMPv3 engine discovery?

Yes. Gufo SNMP automatically discovers the SNMPv3 engine ID when required, so you don't need to configure it manually.

## Performance

### Is Gufo SNMP fast?

Yes. Gufo SNMP is designed to provide the highest practical performance for a Python SNMP library. Performance-critical operations are implemented in Rust, while the Python API remains simple and convenient to use.

### Why is Gufo SNMP implemented in Rust?

Gufo SNMP uses Rust for performance, memory safety, and efficient memory management. Performance-critical operations are implemented in Rust and exposed through a simple, Python-native API. This includes a zero-copy BER parser that minimizes memory allocations and avoids unnecessary data copying while processing SNMP messages, significantly reducing CPU and memory overhead.

### Can Gufo SNMP handle large SNMP walks?

Yes. Gufo SNMP can handle SNMP walks of any practical size. It does not accumulate the entire walk result in memory. Results are returned to the application as they are received, allowing the application to process or store them incrementally.

### Does Gufo SNMP support asynchronous operation?

Yes. Gufo SNMP provides both synchronous and asynchronous APIs, allowing you to use it with standard Python code or asyncio applications.

### Can I run many SNMP requests concurrently?

Yes. Gufo SNMP supports running SNMP requests concurrently using separate sessions. It has been tested with several hundred concurrent sessions, making it suitable for applications communicating with large numbers of devices in parallel.

### How does Gufo SNMP compare to Python SNMP libraries?

Gufo SNMP is designed to be one of the fastest SNMP libraries available for Python. It combines a simple, Python-native API with a Rust-powered core for performance-critical operations and is optimized for working with large numbers of SNMP devices.

### Does Gufo SNMP release the GIL?

Yes. Gufo SNMP releases the Python GIL whenever possible. Performance-critical operations are executed outside the GIL, allowing other Python threads to run concurrently.

### Can Gufo SNMP handle large and heterogeneous networks?

Yes. Large-scale and heterogeneous networks are Gufo SNMP's native environment. It is designed to communicate efficiently with large numbers of devices from different vendors, using different SNMP versions and capabilities.

### Does Gufo SNMP support Python free-threading?

Not yet. We are currently experimenting with Python free-threading support.

### Is Gufo SNMP used in large-scale production environments?

Yes. Gufo SNMP is used in production environments monitoring large-scale networks. Some installations monitor more than one million network devices, demonstrating its ability to operate efficiently at very large scale.

## Python API

### Is Gufo SNMP synchronous or asynchronous?

Both. Gufo SNMP provides separate synchronous and asynchronous APIs, so you can use it with standard Python code or asyncio applications.

### Can I use Gufo SNMP with asyncio?

Yes. Import SnmpSession from `gufo.snmp.aio` and use it with `asyncio` applications.

### Can I use Gufo SNMP from synchronous Python code?

Yes. Import SnmpSession from `gufo.snmp.sync` and use it from regular synchronous Python code.

### How do I create an SNMP session?

Create an SnmpSession with the device address and SNMP credentials. For example:

```python
from gufo.snmp.sync import SnmpSession

with SnmpSession(
    addr="192.0.2.1",
    community="public",
) as session:
    ...
```

For asynchronous applications, use `async with` with `SnmpSession` from `gufo.snmp.aio`:

```python
from gufo.snmp.aio import SnmpSession

async with SnmpSession(
    addr="192.0.2.1",
    community="public",
) as session:
    ...
```

### How do I specify SNMP credentials?

For SNMPv1 and SNMPv2c, specify the community string directly:

```python
with SnmpSession(addr="192.0.2.1", community="public") as session:
    ...
```

For SNMPv3, create a User with the required authentication and privacy keys:

```python
with SnmpSession(
    addr="192.0.2.1",
    user=User("username",
    auth_key=Sha1Key(b"authpass"),
    priv_key=Aes128Key(b"privpass"))
    ) as session:
    ...
```

### How do I specify a timeout?

Set the timeout parameter when creating an SnmpSession. The timeout is specified as a floating-point value in seconds:

```python
with SnmpSession(..., timeout=2.5) as session:
    ...
```

### How do I handle SNMP errors?

Gufo SNMP raises exceptions for SNMP and protocol errors. All Gufo SNMP exceptions inherit from SnmpError, so applications can catch all SNMP-related errors at once or handle specific errors individually.

```python
from gufo.snmp import SnmpError, SnmpAuthError

try:
    with SnmpSession(addr="192.0.2.1", community="public") as session:
        result = session.get("1.3.6.1.2.1.1.1.0")
except SnmpError:
    ...
```

### How are SNMP values represented in Python?

SNMP encodes the type of each value in the response using BER. Gufo SNMP uses this information to automatically convert SNMP values to the corresponding native Python types. See the [Supported BER Types](dev/types.md) section for the complete mapping between BER and Python types.

### How are OIDs represented?

OIDs are represented as strings in Python. For example:

```python
oid = "1.3.6.1.2.1.1.1.0"
```

### Can I reuse an SNMP session for multiple requests?

Yes. An SnmpSession can be reused for multiple sequential requests. For concurrent requests, use a separate session for each concurrent operation.

### Can I use one SNMP session with multiple devices?

No. An SnmpSession is associated with a single SNMP device. Create a separate session for each device you want to communicate with.

## SNMPv3

### How do I configure SNMPv3?

Create a `User` with the required authentication and privacy keys, then pass it to `SnmpSession`:

```python
from gufo.snmp import User, Sha1Key, Aes128Key
from gufo.snmp.sync import SnmpSession

with SnmpSession(
    addr="192.0.2.1",
    user=User(
        "username",
        auth_key=Sha1Key(b"authpass"),
        priv_key=Aes128Key(b"privpass"),
    ),
) as session:
    print(session.get("1.3.6.1.2.1.1.1.0"))
```

### Can I use SNMPv3 without authentication or encryption?

Yes. Simply omit the corresponding authentication and privacy keys when creating the User. For example, to use SNMPv3 without authentication or encryption:

```python
user = User("username")
```

### How can I specify SNMPv3 keys?

Gufo SNMP accepts SNMPv3 keys in three forms: as a password, a master key, or a localized key. You can use whichever form you already have; Gufo SNMP automatically converts it to the form required for communication with the target device.

### What is the difference between a password, a master key, and a localized key?

A **password** is the human-readable secret used to derive an SNMPv3 key. A **master key** is the key derived from the password using the SNMPv3 key derivation algorithm. A **localized key** is derived from the master key and the SNMPv3 engine ID, making it specific to a particular SNMP engine.

### Can I use a pre-localized SNMPv3 key?

Yes. Gufo SNMP accepts localized keys directly. You don't need to convert them back to a password or master key.

### Can Gufo SNMP derive a key from a password?

Yes. Gufo SNMP can derive the required SNMPv3 key from a password automatically. You can provide a password, master key, or localized key, and Gufo SNMP handles the required conversion automatically.

### Is Blumenthal key expansion supported?

Yes. Gufo SNMP supports the Blumenthal key expansion scheme for AES-192 and AES-256 privacy keys.

```python
from gufo.snmp import KeyExpansion, User

user = User(
    "myuser",
    auth_key=...,
    priv_key=...,
    key_expansion=KeyExpansion.Blumenthal,
)
```

### Is Cisco/Reeder key expansion supported?

Yes. Gufo SNMP supports the Cisco/Reeder key expansion scheme for AES-192 and AES-256 privacy keys.

```python
from gufo.snmp import KeyExpansion, User

user = User(
    "myuser",
    auth_key=...,
    priv_key=...,
    key_expansion=KeyExpansion.Cisco,
)
```
### What are noAuthNoPriv, authNoPriv, and authPriv?

These are the three SNMPv3 security levels:

- `noAuthNoPriv` — authentication and encryption are disabled.
- `authNoPriv` — authentication is enabled, but encryption is disabled.
- `authPriv` — both authentication and encryption are enabled.

Gufo SNMP selects the corresponding security level based on the authentication and privacy keys provided in the User.

### What happens if SNMPv3 authentication fails?

If the device responds with an SNMPv3 authentication error, Gufo SNMP raises SnmpAuthError. Some devices may silently discard requests with invalid credentials instead of returning an error; in this case, the request eventually times out.

## Troubleshooting

### Why do I get a timeout?

A timeout means that Gufo SNMP did not receive a response from the device within the configured timeout. Check that the device is reachable, SNMP is enabled and configured correctly, and the address, port, credentials, and SNMP version are correct. Some devices may silently discard requests with invalid SNMPv3 credentials, which also results in a timeout.

### How can I check whether the SNMP agent is reachable?

The simplest way is to send an SNMP request to the device. If the request returns a response, the SNMP agent is reachable. For example:

```python
with SnmpSession(addr="192.0.2.1", community="public", timeout=2.0) as session:
    print(session.get("1.3.6.1.2.1.1.3.0"))
```

If the request times out, check network connectivity, the SNMP agent configuration, and the device address and port.

### Why does GET work but GETBULK fail?

GETBULK is available only in SNMPv2c and SNMPv3. It is not supported by SNMPv1. If GET works but GETBULK fails, check the SNMP version and whether the device supports GETBULK. Some devices also have limitations on the maximum number of variables returned in a single GETBULK response.

### Why does an SNMPv3 request fail authentication?

SNMPv3 authentication can fail if the username, authentication protocol, or authentication key is incorrect. Check that the configured credentials match the user configured on the SNMP agent. Also make sure that the SNMPv3 engine ID has been discovered correctly, as the engine ID is used to localize authentication keys.

### How do I troubleshoot SNMPv3 encryption errors?

Check that the username, privacy protocol, and privacy key match the configuration on the SNMP agent. Gufo SNMP supports DES and AES-128 privacy protocols. If the device silently discards requests with invalid credentials, the request may result in a timeout rather than an explicit error.

### How do I handle an SNMP error-status response?

Gufo SNMP converts SNMP error-status responses into Python exceptions. You can catch SnmpError to handle SNMP errors or catch a specific exception when you need to handle a particular condition.

### How does Gufo SNMP handle noSuchObject?

When an SNMP agent returns `noSuchObject`, Gufo SNMP returns `None` to the application.

### How does Gufo SNMP handle noSuchInstance?

When an SNMP agent returns `noSuchInstance`, Gufo SNMP raises `NoSuchInstance`.

### How does Gufo SNMP handle endOfMibView?

Gufo SNMP handles endOfMibView automatically when walking an SNMP tree. It is used internally to detect the end of the MIB and is not exposed to the application.

## Development

### Where is the Gufo SNMP source code?

The Gufo SNMP source code is available on GitHub. See the [Gufo SNMP repository](https://github.com/gufolabs/gufo_snmp) for the source code, tests, benchmarks, and development tools.

### How is Gufo SNMP structured?

Gufo SNMP is organized into several components, including the Python API and the Rust-powered core. See the [Project's Code Base](dev/codebase.md) for an overview of the project structure and the role of its main components.

### How do I build Gufo SNMP from source?

See the [Building and Testing](dev/testing.md) for instructions on setting up the development environment and building Gufo SNMP from source.

### How do I run the tests?

See the [Building and Testing](dev/testing.md) for instructions on running the Gufo SNMP test suite.

### How do I run the benchmarks?

See the [Python SNMP Clients Benchmarks](benchmarks/index.md) for benchmark results and instructions on running the benchmarks.

### How can I contribute to Gufo SNMP?

See the [Contributing Guide](dev/CONTRIBUTING.md) for information on contributing to Gufo SNMP.

### How can I report a bug?

Report bugs through [GitHub Issues](https://github.com/gufolabs/gufo_snmp/issues). Please include enough information to reproduce the problem.

### How can I request a feature?

Have an idea for Gufo SNMP? Start a discussion in [GitHub Discussions](https://github.com/gufolabs/gufo_snmp/discussions). Tell us what you would like to do, why it matters, and what kind of API or behavior you have in mind.

## About Gufo

### What does "Gufo" mean?

*Gufo* means *the Owl* in Italian.

### Why the owls?

We love owls and the viable parts of our technologies were proven at the project named "the Owl".

### What is "Gufo Labs"?

[Gufo Labs](https://gufolabs.com/) is the Italian company specialized in network and IT consulting and software research.

### What is "Gufo Stack"?

We've extracted core components behind [NOC](https://getnoc.com/) and released them as independent packages, available under the terms of the 3-clause BSD license.

Our software shares common code quality standards and is battle-proven under high load. We hope our key components will help engineers and developers build reliable networks and robust network management software.

See [more details](https://gufolabs.com/products/gufo-stack/).
