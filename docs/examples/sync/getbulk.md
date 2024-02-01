# Gufo SNMP Example: GetBulk Request

We have mastered the iteration of the MIB view
in our [getnext](getnext.md) example. SNMP v2
also offers more effective approach - the GetBulk
request. *Gufo SNMP* hides all implementation
difference and the interface to the GetBulk
requests is almost identical to the GetNext one.

``` py title="getbulk.py" linenums="1"
--8<-- "examples/sync/getbulk.py"
```

Let's see the details.

``` py title="getbulk.py" linenums="1" hl_lines="1"
--8<-- "examples/sync/getbulk.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="getbulk.py" linenums="1" hl_lines="3"
--8<-- "examples/sync/getbulk.py"
```

`SnmpSession` object holds all necessary API. We're using a synchronous
version from `gufo.snmp.sync_client`.


``` py title="getbulk.py" linenums="1" hl_lines="6"
--8<-- "examples/sync/getbulk.py"
```

We define our main function and expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* Base OID to query.

``` py title="getbulk.py" linenums="1" hl_lines="7"
--8<-- "examples/sync/getbulk.py"
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

``` py title="getbulk.py" linenums="1" hl_lines="8"
--8<-- "examples/sync/getbulk.py"
```

We use `SnmpSession.getbulk()` function to iterate within base OID. The function is an
iterator yielding pairs of `(OID, value)`, so we use `for` construction to iterate over the values.
See [SnmpSession.getbulk() reference][gufo.snmp.sync_client.SnmpSession.getbulk]
for further details. 

``` py title="getbulk.py" linenums="1" hl_lines="9"
--8<-- "examples/sync/getbulk.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="getbulk.py" linenums="1" hl_lines="12"
--8<-- "examples/sync/getbulk.py"
```

Lets run our `main()` function
and pass first command-line parameters as address, community, and oid.

## Running

Let's check our script. Run example as:

```
$ python3 examples/sync/getbulk.py 127.0.0.1 public 1.3.6.1.2.1.1
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
