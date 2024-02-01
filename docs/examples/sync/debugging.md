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
--8<-- "examples/sync/debugging.py"
```

Let's see the details.

``` py title="debugging.py" linenums="1" hl_lines="1"
--8<-- "examples/sync/debugging.py"
```

`Snmpd` wrapper should be imported from `gufo.snmp.snmpd` directly.

``` py title="debugging.py" linenums="1" hl_lines="2"
--8<-- "examples/sync/debugging.py"
```

`SnmpSession` object holds all necessary API. We're using a synchronous
version from `gufo.snmp.sync_client`.


``` py title="debugging.py" linenums="1" hl_lines="5"
--8<-- "examples/sync/debugging.py"
```

We define our `main` function. Unlike our [get](get.md), [getmany](getmany.md),
and [getnext](getnext.md) examples we do not expect any external arguments.

``` py title="debugging.py" linenums="1" hl_lines="6"
--8<-- "examples/sync/debugging.py"
```

We need to create `Snmpd` context to run local snmpd instance.
Then we need to create `SnmpSession` object which wraps the client's session.
We using context managers using `with` clause. Refer to the [get](get.md), [getmany](getmany.md),
and [getnext](getnext.md) examples for additional details.

Both `Snmpd` and `SnmpSession` are highly configurable, so refer to the
[Snmpd][gufo.snmp.snmpd.Snmpd] and
[SnmpSession][gufo.snmp.sync_client.SnmpSession]
references.

``` py title="debugging.py" linenums="1" hl_lines="7"
--8<-- "examples/sync/debugging.py"
```

We use `SnmpSession.getnext()` function to iterate within base OID. The function is an
iterator yielding pairs of `(OID, value)`, so we use `for` construction to iterate over the values.
See [SnmpSession.getnext() reference][gufo.snmp.sync_client.SnmpSession.getnext]
for further details. 

``` py title="debugging.py" linenums="1" hl_lines="8"
--8<-- "examples/sync/debugging.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="debugging.py" linenums="1" hl_lines="11"
--8<-- "examples/sync/debugging.py"
```

Lets run our `main()` function.

## Running

Let's check our script. Run example as:

```
$ python3 examples/sync/debugging.py
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
