# Gufo SNMP Example: Debugging

In our previous examples we have relied on existing
and running SNMP agent. But Gufo SNMP offers the useful
`Snmpd` wrapper to configure and run the local instance
of the `snmpd` which can be started and terminated
along your application.

!!! note
    This feature in requires an installed Net-SNMP package.
    Refer to your operation system's manuals for details.

``` py title="debugging.py" linenums="1"
--8<-- "examples/debugging.py"
```

Let's see the details.

``` py title="debugging.py" linenums="1" hl_lines="1"
--8<-- "examples/debugging.py"
```
*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="debugging.py" linenums="1" hl_lines="3"
--8<-- "examples/debugging.py"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp`.

``` py title="debugging.py" linenums="1" hl_lines="4"
--8<-- "examples/debugging.py"
```

`Snmpd` wrapper should be imported from `gufo.snmp.snmpd` directly.

``` py title="debugging.py" linenums="1" hl_lines="7"
--8<-- "examples/debugging.py"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. Unlike our [get](get.md), [getmany](getmany.md),
and [getnext](getnext.md) examples we do not expect any external arguments.

``` py title="debugging.py" linenums="1" hl_lines="8"
--8<-- "examples/debugging.py"
```

We need to create `Snmpd` context to run local snmpd instance.
Then we need to create `SnmpSession` object which wraps the client's session.
Both context manager is asyncronous and can be used within single `async with`
clause. Refer to the [get](get.md), [getmany](getmany.md),
and [getnext](getnext.md) examples for additional details.

Both `Snmpd` and `SnmpSession` are highly configurable, so refer to the
[Snmpd](../../reference/gufo/snmp/client#gufo.snmp.snmpd.Snmpd) and
[SnmpSession](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession)
references.

``` py title="debugging.py" linenums="1" hl_lines="9"
--8<-- "examples/debugging.py"
```

We use `SnmpSession.getnext()` function to iterate within base OID. The function is an asynchronous
iterator returning pairs of `(OID, value)`, so we use `async for` construction to iterate over the values.
See [SnmpSession.getnext() reference](../../reference/gufo/snmp/client#gufo.snmp.client.SnmpSession.getnext)
for further details. 

``` py title="debugging.py" linenums="1" hl_lines="10"
--8<-- "examples/debugging.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="debugging.py" linenums="1" hl_lines="13"
--8<-- "examples/debugging.py"
```

Lets run our asynchronous `main()` function via `asyncio.run`
and pass first command-line parameters as address, community, and oid.

## Running

Let's check our script. Run example as:

```
$ python3 examples/debugging.py
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
