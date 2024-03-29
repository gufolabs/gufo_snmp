# Gufo SNMP Example: Multi Items Get Request

We have mastered the requesting of single item in our [get example](get.md).
But SNMP allows to query multiple keys in single request. Let's consider
the example.

``` py title="getmany.py" linenums="1"
--8<-- "examples/sync/getmany.py"
```

Let's see the details.

``` py title="getmany.py" linenums="1" hl_lines="1"
--8<-- "examples/sync/getmany.py"
```

Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="getmany.py" linenums="1" hl_lines="2"
--8<-- "examples/sync/getmany.py"
```
*Gufo SNMP* is a typed library and it is good practice
to place type hints in your code, so we import
required type hints from Python's `typing` module.

``` py title="getmany.py" linenums="1" hl_lines="4"
--8<-- "examples/sync/getmany.py"
```

`SnmpSession` object holds all necessary API. We're using a synchronous
version from `gufo.snmp.sync_client`.

``` py title="getmany.py" linenums="1" hl_lines="7"
--8<-- "examples/sync/getmany.py"
```

We define our main function and expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* List of OIDs to query.

``` py title="getmany.py" linenums="1" hl_lines="8"
--8<-- "examples/sync/getmany.py"
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

``` py title="getmany.py" linenums="1" hl_lines="9"
--8<-- "examples/sync/getmany.py"
```

We use `SnmpSession.get_many()` function to query multiple OIDs.
See [SnmpSession.get() reference][gufo.snmp.sync_client.SnmpSession.get_many] for further details.

`get_many()` returns a `dict`, where keys are the requested OIDs, and values are the query results.

!!! note

    `get_many()` ignores non-existent OIDs, so it is up to the application to check
    the resulting dict for missed keys.

``` py title="getmany.py" linenums="1" hl_lines="10 11"
--8<-- "examples/sync/getmany.py"
```

It is up to the application how to deal with the result.
In our example we just print all the items.

``` py title="getmany.py" linenums="1" hl_lines="14"
--8<-- "examples/sync/getmany.py"
```

Lets run our `main()` function and pass first command-line parameters as address, community and OIDs.
We use the rest of command line as the list of OIDs to query.

## Running

Let's check our script. Run example as:

```
$ python3 examples/sync/getmany.py 127.0.0.1 public 1.3.6.1.2.1.1.6.0 1.3.6.1.2.1.1.4.0
1.3.6.1.2.1.1.6.0: Gufo SNMP Test
1.3.6.1.2.1.1.4.0: test <me@example.com>
```