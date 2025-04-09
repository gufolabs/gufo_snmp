**SNMP v2c** uses plaintext, non-encrypted BER-encoded messages with simple, community-based authorization.

This benchmark suite evaluates the following aspects of SNMP v2c operations:

- [**GETNEXT**](getnext.md) — Sequential iteration over the entire MIB using
  the **GETNEXT** operation.
- [**GETBULK**](getbulk.md) — Sequential iteration over the entire MIB
  using the **GETBULK** operation.
- [**GETNEXT (Parallel)**](getnext_p.md) — Four parallel sessions performing
  sequential iteration over the entire MIB using the **GETNEXT** operation.
- [**GETBULK (Parallel)**](getbulk_p.md) — Four parallel sessions performing
  sequential iteration over the entire MIB using the **GETBULK** operation.
