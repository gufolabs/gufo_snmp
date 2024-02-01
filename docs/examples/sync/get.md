# Gufo SNMP Example: Single Item Get Request

`Get` is one of the basic SNMP operations allowing to query of the agent 
for one or more management keys. Let's consider the situation of
getting the single key.

``` py title="get.py" linenums="1"
--8<-- "examples/sync/get.py"
```

Let's see the details.

``` py title="get.py" linenums="1" hl_lines="1"
--8<-- "examples/sync/get.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="get.py" linenums="1" hl_lines="3"
--8<-- "examples/sync/get.py"
```

`SnmpSession` object holds all necessary API. We're using a synchronous
version from `gufo.snmp.sync_client`.

``` py title="get.py" linenums="1" hl_lines="6"
--8<-- "examples/sync/get.py"
```

We define our main function and expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* OID to query.

``` py title="get.py" linenums="1" hl_lines="7"
--8<-- "examples/sync/get.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as context manager
using the `with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference][gufo.snmp.sync_client.SnmpSession]
for further details. In our example, we set the agent's address and SNMP community
to the given values.

``` py title="get.py" linenums="1" hl_lines="8"
--8<-- "examples/sync/get.py"
```

We use `SnmpSession.get()` function to query OID. See [SnmpSession.get() reference][gufo.snmp.sync_client.SnmpSession.get] for further details.

``` py title="get.py" linenums="1" hl_lines="9"
--8<-- "examples/sync/get.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="get.py" linenums="1" hl_lines="12"
--8<-- "examples/sync/get.py"
```

Lets run our `main()` function pass first command-line parameters as address, community, and OID.

## Running

Let's check our script. Run example as:

```
$ python3 examples/sync/get.py 127.0.0.1 public 1.3.6.1.2.1.1.6.0
Gufo SNMP Test
```