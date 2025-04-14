Here is the summary table for Gufo SNMP bencmarks.

--8<-- "docs/benchmarks/conclusions.txt"

**Conclusions:**

* **Gufo SNMP is clear winner:** in the terms of performance.
* **Async mode** adds significant overhead per each I/O operation. It is expecially
  noticeable in **GETNEXT** mode.
* **GETBULK** consistently outperforms **GETNEXT**. As anticipated, it provides
  better performance and should be preferred whenever supported by equipment.
* **The encryption overhead of SNMPv3** (AES128 + SHA1) is minimal,
  showing little impact on performance.
* **Gufo SNMP demonstrates good scalability:** running four parallel tasks
  takes only about 1.5× the time of a single task, indicating efficient performance
  even beyond Python's GIL limitations.
