# Benchmarks

## Rust Core Benchmarks

Gufo SNMP uses [Iai][Iai], a [Rust][Rust] [Cachegrind][Cachegrind]-based
bencmark framework to estimate performance of the critical code paths.

| Name                    | Inst.[^1] | L1 Acc.[^2] | L2 Acc.[^3] | RAM Acc.[^4] | Est. Cycles [^5] |
| ----------------------- | --------: | ----------: | ----------: | -----------: | ---------------: |
| decode_header           |        56 |          72 |           2 |            8 |              362 |
| decode_getresponse      |      5045 |        7038 |          12 |          103 |            10703 |
| decode_bool             |       123 |         169 |           3 |           14 |              674 |
| decode_counter32        |       154 |         203 |           2 |           15 |              738 |
| decode_counter64        |       154 |         201 |           3 |           16 |              776 |
| decode_gauge32          |       154 |         201 |           3 |           16 |              776 |
| decode_int              |       164 |         212 |           2 |           17 |              817 |
| decode_ipaddress        |       132 |         181 |           3 |           15 |              721 |
| decode_null             |       113 |         153 |           3 |           13 |              623 |
| decode_objectdescriptor |       118 |         165 |           2 |           14 |              665 |
| decode_oid              |       478 |         655 |           3 |           21 |             1405 |
| decode_octetstring      |       136 |         186 |           4 |           14 |              696 |
| decode_opaque           |       136 |         187 |           3 |           14 |              692 |
| decode_real_nr1         |       305 |         380 |           6 |           24 |             1250 |
| decode_real_nr2         |       484 |         600 |           6 |           41 |             2065 |
| decode_real_nr3         |       460 |         573 |           7 |           41 |             2043 |
| decode_relative_oid     |       388 |         534 |           3 |           20 |             1249 |
| decode_timeticks        |       170 |         224 |           3 |           13 |              694 |
| decode_uinteger32       |       170 |         224 |           2 |           14 |              724 |
| encode_get              |      2968 |        3936 |          10 |           79 |             6751 |

[^1]: CPU instructions performed.
[^2]: L1 cache accesses.
[^3]: L2 cache accesses.
[^4]: RAM Accesses.
[^5]: Estimated CPU cycles.
[Iai]: https://github.com/bheisler/iai
[Rust]: https://www.rust-lang.org
[Cachegrind]: https://valgrind.org/docs/manual/cg-manual.html