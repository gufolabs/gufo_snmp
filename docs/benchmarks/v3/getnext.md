Perform SNMP v3 GETNEXT requests to iterate through whole MIB.
Use SHA-1 hasing and AES-128 encryption. This test evaluates:

* The efficiency of the network stack.
* The efficiency of BER encoder and decoder.
* The efficiency of the BER-to-Python data types mapping.
* The efficiency of the crypto stack.

Look at the [source code][source] for details.

!!! notes

    * easysnmp doesn't supports async mode

Run tests:

```
pytest benchmarks/test_v3_getnext.py
```

**Results (lower is better)**

```
--8<-- "docs/benchmarks/v3/.txt"test_v3_getnext
```

![Median chart](getnext.png)
*Lower is better*

[source]: https://github.com/gufolabs/gufo_snmp/blob/master/benchmarks/test_v3_getnext.py