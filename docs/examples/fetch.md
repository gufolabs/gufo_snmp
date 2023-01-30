# Gufo SNMP Example: Fetch

We have mastered the iteration of the MIB view
in our [getnext](getnext.md) and [getbulk](getbulk.md) examples.
*Gufo SNMP* also offers a convenient wrapper to combine
them into the single `.fetch()` wrapper. This may be useful
when the application combines SNMP v1 and SNMP v2c queries
and it is desirable to hide such implementation details.

``` py title="fetch.py" linenums="1"
--8<-- "examples/fetch.py"
```

Let's see the details.

``` py title="fetch.py" linenums="1" hl_lines="1"
--8<-- "examples/fetch.py"
```
*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="fetch.py" linenums="1" hl_lines="2"
--8<-- "examples/fetch.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="fetch.py" linenums="1" hl_lines="4"
--8<-- "examples/fetch.py"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp`.

``` py title="fetch.py" linenums="1" hl_lines="7"
--8<-- "examples/fetch.py"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. We expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* Base OID to query.

``` py title="fetch.py" linenums="1" hl_lines="8 9 10"
--8<-- "examples/fetch.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as async context manager
with the `async with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

We can use `allow_bulk` parameter to enable bulk requests whenever the protocol
version allows it or to deny bulk requests in any case.

| Version | False   | True    |
| ------- | ------- | ------- |
| v1      | getnext | getnext |
| v2c     | getnext | getnext |

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession)
for further details. In our example, we set the agent's address and SNMP community
to the given values.

``` py title="fetch.py" linenums="1" hl_lines="11"
--8<-- "examples/fetch.py"
```

We use `SnmpSession.fetch()` function to iterate within base OID just like the
`SnmpSession.getnext()` and `SnmpSession.getbulk()`.

The function is an asynchronous
iterator returning pairs of `(OID, value)`, so we use `async for` construction to iterate over the values.
See [SnmpSession.getbulk() reference](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession.getbulk)
for further details. 

``` py title="fetch.py" linenums="1" hl_lines="12"
--8<-- "examples/fetch.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="fetch.py" linenums="1" hl_lines="15"
--8<-- "examples/fetch.py"
```

Lets run our asynchronous `main()` function via `asyncio.run`
and pass first command-line parameters as address, community, and oid.

## Running

Let's check our script. Run example as:

```
$ python3 examples/fetch.py 127.0.0.1 public 1.3.6.1.2.1.1
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
