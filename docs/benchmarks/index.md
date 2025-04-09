# Python SNMP Clients Benchmarks
!!! warning "Disclaimer"

    All following information is provided only for reference.
    These tests are performed by [Gufo Labs][Gufo Labs] to estimate the performance
    of [Gufo SNMP][Gufo SNMP] against major competitors, so they cannot be considered
    independent and unbiased.

!!! note

    Although performance is an absolute requirement for [Gufo Stack][Gufo Stack],
    other factors such as maturity, community, features, examples, and existing code base
    should also be considered.

This benchmark evaluates several Python SNMP client libraries:

* [Gufo SNMP][Gufo SNMP] (current version)
* [pysnmp][pysnmp] (7.1.17)

The evaluation covers the following aspects:

* Performance in synchronous (blocking) mode, if supported.
* Performance in asynchronous (non-blocking) mode, if supported.
* Performance in plain-text SNMP (v2c) and encrypted (SNMP v3) modes.
* Ability to release GIL in multi-threaded applications.

All benchmarks are performed against a local Net-SNMPd installation
using wrapper, provided by `gufo.snmp.snmpd`.

The benchmarking environment utilizes an docker container running on
Apple M4 Pro processor.

## Benchmark Results

* [Preparing](preparing.md)
* [SNMP v2c](v2c/index.md)
* [SNMP v3](v3/index.md)
* [Conclustions](conclusions.md)
* [Feedback](feedback.md)

[Gufo Labs]: https://gufolabs.com/
[Gufo Stack]: https://docs.gufolabs.com/
[Gufo SNMP]: https://docs.gufolabs.com/gufo_snmp/
[pysnmp]: https://docs.lextudio.com/snmp/