# Gufo SNMP Example: Rate Limiting

We have mastered the various ways of querying the MIB view
in our [get](get.md), [getnext](getnext.md), [getbulk](getbulk.md),
and [fetch](fetch.md) examples. But in real life, the aggressive
SNMP polling may impose a great burden on the equipment's CPU and
may lead to service degradation or, even, the loss
of connectivity. Likely, *Gufo SNMP* offers the easy way to limit
the rate of the outgoing requests to adjust the impact. 

``` py title="ratelimit.py" linenums="1"
--8<-- "examples/sync/ratelimit.py"
```

Let's see the details.

``` py title="ratelimit.py" linenums="1" hl_lines="1"
--8<-- "examples/sync/ratelimit.py"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.

``` py title="ratelimit.py" linenums="1" hl_lines="3"
--8<-- "examples/sync/ratelimit.py"
```

`SnmpSession` object holds all necessary API. We're using a synchronous
version from `gufo.snmp.sync_client`.

``` py title="ratelimit.py" linenums="1" hl_lines="6"
--8<-- "examples/sync/ratelimit.py"
```

We define our main function and expect the following arguments:

* Address of the agent.
* SNMP community to authorize.
* Base OID to query.

``` py title="ratelimit.py" linenums="1" hl_lines="7 8 9"
--8<-- "examples/sync/ratelimit.py"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as context manager
using the `with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

We can use `allow_bulk` parameter to enable bulk requests whenever the protocol
version allows it or to deny bulk requests in any case. See [fetch](fetch.md) examples
for details.

The only difference from [fetch](fetch.md) example is the `limit_rps` option which
urges *Gufo SNMP* to limit outgoing requests. The parameter holds the limit of the
outgoing requests per second. In out example we will send no more than 10 requests
per second while iterating over the MIB. If you need to send every request in several
seconds, use the fractions. i.e. `0.25` means *one request per each 4 seconds*.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference][gufo.snmp.sync_client.SnmpSession]
for further details. In our example, we set the agent's address and SNMP community
to the given values.

``` py title="ratelimit.py" linenums="1" hl_lines="10"
--8<-- "examples/sync/ratelimit.py"
```

We use `SnmpSession.fetch()` function to iterate within base OID just like the
`SnmpSession.getnext()` and `SnmpSession.getbulk()`.

The function is an
iterator yieldig pairs of `(OID, value)`, so we use `for` construction to iterate over the values.
See [SnmpSession.getbulk() reference][gufo.snmp.sync_client.SnmpSession.getbulk]
for further details. 

``` py title="ratelimit.py" linenums="1" hl_lines="11"
--8<-- "examples/sync/ratelimit.py"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="ratelimit.py" linenums="1" hl_lines="14"
--8<-- "examples/sync/ratelimit.py"
```

Lets run our `main()` function and pass first command-line parameters as address, community, and oid.

## Running

Let's check our script. Run example as:

```
$ python3 examples/sync/ratelimit.py 127.0.0.1 public 1.3.6.1.2.1.1
1.3.6.1.2.1.1.1.0: b'Linux d280d3a0a307 5.15.49-linuxkit #1 SMP Tue Sep 13 07:51:46 UTC 2022 x86_64'
1.3.6.1.2.1.1.2.0: 1.3.6.1.4.1.8072.3.2.10
1.3.6.1.2.1.1.3.0: 36567296
1.3.6.1.2.1.1.4.0: b'test <test@example.com>'
1.3.6.1.2.1.1.5.0: b'd280d3a0a307'
1.3.6.1.2.1.1.6.0: b'Gufo SNMP Test'
1.3.6.1.2.1.1.7.0: 72
...
```
