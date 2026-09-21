# gufo-snmp — SNMP Client Utility

`gufo-snmp` is a command-line SNMP client for querying SNMP agents. It provides a compact interface for common SNMP operations and closely resembles the command-line tools from the Net-SNMP family.

## Usage

```text
usage: gufo-snmp [-h] [--version {v1,v2c,v3}] [-v1 | -v2c | -v3]
                 [--command {GET,GETNEXT,GETBULK}] [-p PORT]
                 [-c COMMUNITY] [-u USER]
                 [-a {MD5,SHA,SHA224,SHA256,SHA384,SHA512}] [-A AUTH_PASS]
                 [-x {DES,AES,AES128,AES192,AES256}] [-X SECURITY_PASS]
                 [-O OFLAGS]
                 address ...

SNMP Client

positional arguments:
  address               Agent
  oids                  OIDs

options:
  -h, --help            show this help message and exit

  --version {v1,v2c,v3}
                        SNMP protocol version

  -v1                   SNMP v1
  -v2c                  SNMP v2c
  -v3                   SNMP v3

  --command {GET,GETNEXT,GETBULK}
                        SNMP command

  -p, --port PORT       Agent port (default: 161)

  -c, --community COMMUNITY
                        Community string (v1/v2c)

  -u, --user USER       User name (v3)

  -a, --auth-protocol {MD5,SHA,SHA224,SHA256,SHA384,SHA512}
                        Authentication protocol (v3)

  -A, --auth-pass AUTH_PASS
                        Authentication pass-phrase (v3)

  -x, --security-protocol {DES,AES,AES128,AES192,AES256}
                        Privacy protocol (v3)

  -X, --security-pass SECURITY_PASS
                        Privacy pass-phrase (v3)

  -O OFLAGS             Output formatting flags (may be repeated or combined)
                        Supported flags:

                          a : print all strings in ASCII format
                          x : print all strings in hexadecimal format
                          q : quick output with a space separator
                          Q : quick output with an equal-sign separator
                          T : print human-readable text along with hex strings
                          v : print values only (not OID = value)
```

## Commands

### GET

Perform a GET request.

If exactly one OID is specified, `GET` is selected automatically.

```text
gufo-snmp 192.0.2.1 1.3.6.1.2.1.1.1.0
```

### GETMANY

Perform a GET request for multiple OIDs.

When `GET` is selected with multiple OIDs, `gufo-snmp` uses a single multi-OID request.

```text
gufo-snmp 192.0.2.1 1.3.6.1.2.1.1.1.0 1.3.6.1.2.1.1.3.0
```

### GETNEXT

Perform a GETNEXT request for each specified OID.

```text
gufo-snmp --command GETNEXT 192.0.2.1 1.3.6.1.2.1.1
```

### GETBULK

Perform a GETBULK request for each specified OID.

`GETBULK` is not available with SNMPv1.

```text
gufo-snmp --command GETBULK 192.0.2.1 1.3.6.1.2.1.2
```

## SNMP Versions

### SNMPv1 / SNMPv2c

Use `-c` or `--community` to specify the community string.

```text
gufo-snmp -v2c -c public 192.0.2.1 1.3.6.1.2.1.1.1.0
```

SNMPv2c is the default protocol version.

### SNMPv3

Use `-u` to specify the USM user. Authentication and privacy can be configured with `-a`, `-A`, `-x`, and `-X`.

For example:

```text
gufo-snmp -v3 -u monitor \
    -a SHA256 -A authpass \
    -x AES256 -X privpass \
    192.0.2.1 1.3.6.1.2.1.1.1.0
```

An authentication protocol and pass-phrase must be specified together. Likewise, a privacy protocol and pass-phrase must be specified together.

Privacy requires authentication.

## Output Formats

The `-O` option controls output formatting. Multiple flags may be combined, and `-O` may be specified multiple times.

### ASCII (`-Oa`)

Print strings as ASCII, replacing non-printable characters with dots.

Example:

```text
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test
```

### HEX (`-Ox`)

Print strings in hexadecimal format.

Example:

```text
1.3.6.1.2.1.1.6.0 = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74
```

### ASCII + HEX (`-OT`)

Print the human-readable ASCII representation followed by the hexadecimal representation.

Example:

```text
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74
```

### Quick Output (`-Oq`)

Print the OID and value separated by a space instead of `=`.

Example:

```text
1.3.6.1.2.1.1.6.0 Gufo SNMP Test
```

### Equal-Sign Output (`-OQ`)

Print the OID and value separated by `=`.

Example:

```text
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test
```

This is the default separator.

### Value Only (`-Ov`)

Print only the value, without the OID.

Example:

```text
Gufo SNMP Test
```

## Output Flag Combinations

Output flags can be combined. For example:

```text
gufo-snmp -Oxv 192.0.2.1 1.3.6.1.2.1.1.6.0
```

prints the value in hexadecimal without the OID.

The last formatting option affecting the same setting takes effect. For example, `-Oaq` selects ASCII formatting with a space separator.

## Authentication Protocols

The following SNMPv3 authentication protocols are supported:

* MD5
* SHA
* SHA224
* SHA256
* SHA384
* SHA512

## Privacy Protocols

The following SNMPv3 privacy protocols are supported:

* DES
* AES
* AES128
* AES192
* AES256

`AES` is an alias for `AES128`.

## Exit Status

`gufo-snmp` returns zero on successful completion and a non-zero exit code on failure.
