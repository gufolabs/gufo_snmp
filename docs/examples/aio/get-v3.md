# Gufo SNMP Example: SNMPv3 Get Request

In the [previous example](get.md), we demonstrated how to request a single item using SNMP v2c 
in *Gufo SNMP*. Now, we'll show you how to achieve the same with SNMP v3, 
which offers a similar API with additional authentication options.

Despite SNMP v3's increased complexity, *Gufo SNMP* effectively handles all the intricacies,
 making SNMP v3 operations as straightforward as v2c. 
 Let's modify our [previous example](get.md) to utilize SNMP v3.

``` py title="get.py" linenums="1"
--8<-- "examples/aio/get-v3.py"
```

Let's see the details.

``` py title="get.py" linenums="1" hl_lines="1"
--8<-- "examples/aio/get-v3.py::15"
```
*Gufo SNMP* is an async library. In our case
we should run the client from our synchronous script,
so we need to import `asyncio` to use `asyncio.run()`.

``` py title="get.py" linenums="1" hl_lines="2"
--8<-- "examples/aio/get-v3.py::15"
```
Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argparse` or alternatives
    in real-world applications.

``` py title="get.py" linenums="1" hl_lines="4 5"
--8<-- "examples/aio/get-v3.py::15"
```

`SnmpSession` object holds all necessary API, so import it from `gufo.snmp.aio`.
We also need to import `User` class and key algorithm helpers.

``` py title="get.py" linenums="1" hl_lines="7 8 9 10"
--8<-- "examples/aio/get-v3.py::15"
```

SNMPv3 offers various authentication options, so we define mappings
between human-readable names and *Gufo SNMP* key wrappers to use later
in the `get_user` function.

``` py title="get.py" linenums="1" hl_lines="12 13 14 15"
--8<-- "examples/aio/get-v3.py::15"
```
Similarly, SNMPv3 offers various privacy options, and we create mappings 
between human-readable names and key wrappers for these privacy options. 

``` py title="get.py" linenums="18" hl_lines="1"
--8<-- "examples/aio/get-v3.py:18:30"
```

While SNMP v2c relies on a simple community string for authentication, 
SNMPv3 introduces the more intricate User-Based Security Model (USM). 
In this model, a user typically consists of a username,
along with optional authentication and privacy options.
*Gufo SNMP* encapsulates these details within the `User` class.

To facilitate the configuration process, we define the `get_user` function. 
This function processes command-line arguments and returns an instance of the `User` class.

``` py title="get.py" linenums="18" hl_lines="2"
--8<-- "examples/aio/get-v3.py:18:30"
```
We get the user name from the second command-line positional parameter.

``` py title="get.py" linenums="18" hl_lines="3"
--8<-- "examples/aio/get-v3.py:18:30"
```
Authentication options are optional, so we check the fourth command-line
parameter.

``` py title="get.py" linenums="18" hl_lines="4 5"
--8<-- "examples/aio/get-v3.py:18:30"
```
If an authentication option is set, it must have the format `<alg>:<key>`,
where:

* `<alg>` - authentication algorithm, which must be one of `AUTH_ALG` keys.
* `<key>` - an authentication key.

!!! note

    SNMPv3 introduces three forms of keys:

        * Password
        * Master key
        * Localized key

    Choose the form deliberately. *Gufo SNMP* supports all three forms,
    which can be selected through optional parameters of the `*Key` classes.
    This example uses the default form: a password.

Then we find a proper key class via `AUTH_ALG` mapping
and pass a key.

!!! note

    All keys in *Gufo SNMP* are passed as `bytes`, so
    we use `.encode()` method to convert from `str`.

``` py title="get.py" linenums="18" hl_lines="6 7"
--8<-- "examples/aio/get-v3.py:18:30"
```

If no authentication key is provided, set it to `None`
to disable authentication.

``` py title="get.py" linenums="18" hl_lines="8 9 10 11 12"
--8<-- "examples/aio/get-v3.py:18:30"
```
Privacy settings are handled just like authentication settings. We expect
them in the fifth command-line parameter,
and then use `PRIV_ALG` mapping to get a proper algorithm.

As with authentication, `None` means no encryption.

``` py title="get.py" linenums="18" hl_lines="13"
--8<-- "examples/aio/get-v3.py:18:30"
```
Then we construct and return a `User` instance.


``` py title="get.py" linenums="33" hl_lines="1"
--8<-- "examples/aio/get-v3.py:33:36"
```

Asynchronous code must be executed in the asynchronous functions or coroutines.
So we define our function as `async`. We expect the following arguments:

* Address of the agent.
* `User` instance.
* OID to query.

``` py title="get.py" linenums="33" hl_lines="2"
--8<-- "examples/aio/get-v3.py:33:36"
```

First, we need to create `SnmpSession` object which wraps the client's session.
The `SnmpSession` may be used as an instance directly or operated as async context manager
with the `async with` clause. When used as a context manager,
the client automatically closes all connections on the exit of context,
so its lifetime is defined explicitly.

`SnmpSession` constructor offers lots of configuration variables for fine-tuning. Refer to the 
[SnmpSession reference][gufo.snmp.aio.SnmpSession]
for further details. In our example, we set the agent's address and `User`
instance.

``` py title="get.py" linenums="33" hl_lines="3"
--8<-- "examples/aio/get-v3.py:33:36"
```

We use `SnmpSession.get()` function to query OID. The function is asynchronous and
must be awaited. See [SnmpSession.get() reference][gufo.snmp.aio.SnmpSession.get] for further details.

``` py title="get.py" linenums="33" hl_lines="4"
--8<-- "examples/aio/get-v3.py:33:36"
```

It is up to the application how to deal with the result.
In our example we just print it.

``` py title="get.py" linenums="39" hl_lines="1"
--8<-- "examples/aio/get-v3.py:39:39"
```

Let's run our asynchronous `main()` function via `asyncio.run`.
Pass the address as the first command-line parameter, construct the user with
`get_user()`, and pass the OID as the third parameter.

## Running

Let's check our script. Run example as:

```
$ python3 examples/aio/get-v3.py 127.0.0.1 public 1.3.6.1.2.1.1.6.0 sha:12345678 aes128:87654321
Gufo SNMP Test
```
