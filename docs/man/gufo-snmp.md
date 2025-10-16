# gufo-snmp - SNMP Client Utility

`gufo-snmp` is a swiss-army knife for SNMP request which close resembles
Net-SNMP family of CLI tools.

## Usage

```
usage: gufo-snmp [-h] [--version {v1,v2c,v3}] [-v1 | -v2c | -v3] [--command {GET,GETNEXT,GETBULK}] [-p PORT]
                 [-c COMMUNITY] [-u USER] [-a {MD5,SHA}] [-A AUTH_PASS] [-x {DES,AES}] [-X SECURITY_PASS] [-O OFLAGS]
                 address ...

SNMP Client

positional arguments:
  address               Agent
  oids                  OIDs

options:
  -h, --help            show this help message and exit
  --version {v1,v2c,v3}
                        SNMP Protocol version
  -v1                   SNMP v1
  -v2c                  SNMP v2c
  -v3                   SNMP v3
  --command {GET,GETNEXT,GETBULK}
                        Command
  -p, --port PORT       Argent port
  -c, --community COMMUNITY
                        Community (v1/v2c)
  -u, --user USER       User name (v3)
  -a, --auth-protocol {MD5,SHA}
                        Set authentication protocol (v3)
  -A, --auth-pass AUTH_PASS
                        Set authentication protocol pass-phrase (v3)
  -x, --security-protocol {DES,AES}
                        Set security protocol (v3)
  -X, --security-pass SECURITY_PASS
                        Set security protocol pass-phrase (v3)
  -O OFLAGS             Output formatting flags (may be repeated or combined)
                        Supported flags:
                          a : print all strings in ascii format
                          x : print all strings in hex format
                          q : quick print for easier parsing
                          Q : quick print with equal-signs
                          T : print human-readable text along with hex strings
                          v : print values only (not OID = value)
```

## Output Formats

### ASCII (-Oa)

Print strings as text, replacing non-printable characters with dots.

*Example output*:
```
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test
```

### HEX (-Ox)

Print strings in hexadecimal format.

*Example output*:
```
1.3.6.1.2.1.1.6.0 = 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74
```

### ASCII + HEX (-OT)

Print string in human-readable ASCII along with hexadecimal format.

*Example output*:
```
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test 47 75 66 6F 20 53 4E 4D 50 20 54 65 73 74
```

### Without Separator (-Qq)

Do not print `=` between oid and value.

*Example output*:
```
1.3.6.1.2.1.1.6.0 Gufo SNMP Test
```

### With Separator (-QO)

Print `=` between oid and value.

*Example output*:
```
1.3.6.1.2.1.1.6.0 = Gufo SNMP Test
```

### Value Only (-Ov)

Do not print oid.

*Example output*:
```
Gufo SNMP Test
```
