Here is the summary table for Gufo SNMP bencmarks.

--8<-- "docs/benchmarks/conclusions.txt"

**Conclusions:**

* **Async mode** is approximately 10-20% slower than synchronous mode in most cases,
  depending on amount of the network operations.
  This overhead is expected due to the increased complexity of event loop coordination.
* **GETBULK** consistently outperforms GETNEXT. As anticipated, it provides
  better performance and should be preferred whenever supported.
* **The encryption overhead of SNMPv3** (AES128 + SHA1) is minimal,
  showing little impact on performance.
* **Gufo SNMP demonstrates good scalability:** running four parallel tasks
  takes only about 1.5× the time of a single task, indicating efficient performance
  even beyond Python's GIL limitations.
