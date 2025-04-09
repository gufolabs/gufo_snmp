**SNMP v3** introduces the following operation modes:

- **Plaintext** — Matches SNMP v2c but introduces a User Security Model (USM).
- **Integrity Protection** — Protects messages from tampering using a hash-based signature.
- **Privacy Protection** — Encrypts messages to ensure confidentiality.

This benchmark suite focuses on the efficiency of cryptographic operations using
AES-128 and SHA-1 modes, and evaluates the following aspects of SNMP v3 operations:

- [**GETNEXT**](getnext.md) — Sequential iteration over the entire MIB using
  the **GETNEXT** operation.
- [**GETBULK**](getbulk.md) — Sequential iteration over the entire MIB
  using the **GETBULK** operation.
- [**GETNEXT (Parallel)**](getnext_p.md) — Four parallel sessions performing
  sequential iteration over the entire MIB using the **GETNEXT** operation.
- [**GETBULK (Parallel)**](getbulk_p.md) — Four parallel sessions performing
  sequential iteration over the entire MIB using the **GETBULK** operation.
