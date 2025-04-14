Here is the summary table for Gufo SNMP bencmarks.

--8<-- "docs/benchmarks/conclusions.txt"

**Conclusions:**

* **Gufo SNMP is the clear winner** in terms of performance.  
* **Async mode** adds significant overhead to each I/O operation. This is especially noticeable in **GETNEXT** mode.  
* **GETBULK** consistently outperforms **GETNEXT**. As expected, it delivers better performance and should be preferred whenever supported by the equipment.  
* **The encryption overhead of SNMPv3** (AES128 + SHA1) is minimal, with little impact on overall performance.  
* **Gufo SNMP demonstrates good scalability:** running four parallel tasks takes only about 1.5× the time of a single task, indicating efficient performance even beyond Python’s GIL limitations.  
* **BER parsing** is a complex algorithmic operation, so native CPU implementations provide significant performance gains.  
* **Purpose-tailored BER parsers** that map directly to Python types offer substantial advantages over generic SNMP implementations.  
* **Complex abstractions** are slow. A lean and efficient API is key to high performance.
* **Wrappers over C-libraries** may demonstrate an unexpected behaviour in multi-threaded applications.