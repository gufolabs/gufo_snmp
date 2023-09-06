# Gufo SNMP Example: Single Item Get Request

`Get` is one of the basic SNMP operations allowing to query of the agent 
for one or more management keys. Let's consider the situation of
getting the single key.

``` py title="get.py" linenums="1"
--8<-- "examples/get.py"
```

Let's see the details.

``` py title="get.py" linenums="1" hl_lines="1"
--8<-- "examples/get.py"
```
*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="get.py" linenums="1" hl_lines="2"
--8<-- "examples/get.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="get.py" linenums="1" hl_lines="4"
--8<-- "examples/get.py"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp`.

``` py title="get.py" linenums="1" hl_lines="7"
--8<-- "examples/get.py"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. We expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* OID to query.

``` py title="get.py" linenums="1" hl_lines="8"
--8<-- "examples/get.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as async context manager
with the `async with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference][gufo.snmp.client.SnmpSession]
for further details. In our example, we set the agent's address and SNMP community
to the given values.

``` py title="get.py" linenums="1" hl_lines="9"
--8<-- "examples/get.py"
```

We use `SnmpSession.get()` function to query OID. The function is asynchronous and
must be awaited. See [SnmpSession.get() reference][gufo.snmp.client.SnmpSession.get] for further details.

``` py title="get.py" linenums="1" hl_lines="10"
--8<-- "examples/get.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="get.py" linenums="1" hl_lines="13"
--8<-- "examples/get.py"
```

Lets run our asynchronous `main()` function via `asyncio.run`
and pass first command-line parameters as address, community, and OID.

## Running

Let's check our script. Run example as:

```
$ python3 examples/get.py 127.0.0.1 public 1.3.6.1.2.1.1.6.0
Gufo SNMP Test
```