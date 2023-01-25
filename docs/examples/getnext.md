# Gufo SNMP Example: GetNext Request

We have mastered the requesting of single or multiple keys
in our [get](get.md) and [getmany](getmany.md) examples.
The SNMP also defines the way of retrieving all keys under
the given OID - namely the GetNext request.

``` py title="getnext.py" linenums="1"
--8<-- "examples/getnext.py"
```

Let's see the details.

``` py title="getnext.py" linenums="1" hl_lines="1"
--8<-- "examples/getnext.py"
```
*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="getnext.py" linenums="1" hl_lines="2"
--8<-- "examples/getnext.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="getnext.py" linenums="1" hl_lines="4"
--8<-- "examples/getnext.py"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp`.

``` py title="getnext.py" linenums="1" hl_lines="7"
--8<-- "examples/getnext.py"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. We expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* Base OID to query.

``` py title="getnext.py" linenums="1" hl_lines="8"
--8<-- "examples/getnext.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as async context manager
with the `async with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession)
for further details. In our example, we set the agent's address and SNMP community
to the given values.

``` py title="getnext.py" linenums="1" hl_lines="9"
--8<-- "examples/getnext.py"
```

We use `SnmpSession.getnext()` function to iterate within base OID. The function is an asynchronous
iterator returning pairs of `(OID, value)`, so we use `async for` construction to iterate over the values.
See [SnmpSession.getnext() reference](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession.getnext)
for further details. 

``` py title="getnext.py" linenums="1" hl_lines="10"
--8<-- "examples/getnext.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="getnext.py" linenums="1" hl_lines="13"
--8<-- "examples/getnext.py"
```

Lets run our asynchronous `main()` function via `asyncio.run`
and pass first command-line parameters as address, community, and oid.

## Running

Let's check our script. Run example as:

```
$ python3 examples/getnext.py 127.0.0.1 public 1.3.6.1.2.1.1
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
