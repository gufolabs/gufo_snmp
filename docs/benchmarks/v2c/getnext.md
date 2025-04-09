Perform SNMP v2c GETNEXT requests to iterate through whole MIB. This test evaluates:

* The efficiency of the network stack.
* The efficiency of BER encoder and decoder.
* The efficiency of the BER-to-Python data types mapping.

Run tests:

```
pytest benchmarks/test_v2c_getnext.py
```

**Results (lower is better)**

```
--8<-- "docs/benchmarks/v2c/test_v2c_getnext.txt"
```

![Median chart](getnext.png)
*Lower is better*