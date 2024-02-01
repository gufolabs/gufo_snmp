# SNMPv3 Engine ID Discovery

SNMP v3 introduces the concept of the Engine ID, a unique identifier for each SNMP agent 
in the network.

**Gufo SNMP** automatically performs Engine ID discovery as needed. 
However, you can retrieve the actual value for various purposes, 
such as inventory management or performance optimization, 
and use it to skip the discovery step later.

``` py title="engine-id-discovery.py" linenums="1"
--8<-- "examples/async/engine-id-discovery.py"
```

Let's see the details.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="1"
--8<-- "examples/async/engine-id-discovery.py"
```

*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="2"
--8<-- "examples/async/engine-id-discovery.py"
```

Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="4"
--8<-- "examples/async/engine-id-discovery.py"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp`.
For SNMPv3 we also need an `User` class.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="7"
--8<-- "examples/async/engine-id-discovery.py"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. We expect the following arguments:

* Address of the agent.
* Valid user name.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="8"
--8<-- "examples/async/engine-id-discovery.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as async context manager
with the `async with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference][gufo.snmp.async_client.SnmpSession]
for further details. In our example, we set the agent's address and create
SNMPv3 user with default settings.

!!! note
    To perform Engine ID discovery, the only mandatory parameter is the username. 
    Authentication and privacy settings can be left at their default values.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="9"
--8<-- "examples/async/engine-id-discovery.py"
```
`SnmpSession.get_engine_id()` returns discovered Engine Id as bytes.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="10"
--8<-- "examples/async/engine-id-discovery.py"
```
Now, we print the collected Engine ID. Since it is of bytes type, 
we convert the output to hexadecimal form, which is commonly used 
in network equipment configuration.

``` py title="engine-id-discovery.py" linenums="1" hl_lines="13"
--8<-- "examples/async/engine-id-discovery.py"
```

Lets run our asynchronous `main()` function via `asyncio.run`
and pass first command-line parameters as address, community, and OID.

## Running

Let's check our script. Run example as:

```
$ python3 examples/async/engine-id-discovery.py 127.0.0.1 user1
8000b85c03ec02732921c0
```