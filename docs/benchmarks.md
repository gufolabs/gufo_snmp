# Benchmarks

## Rust Core Benchmarks

Gufo SNMP uses [Iai][Iai], a [Rust][Rust] [Cachegrind][Cachegrind]-based
bencmark framework to estimate performance of the critical code paths.

| Name                    | Inst.[^1] | L1 Acc.[^2] | L2 Acc.[^3] | RAM Acc.[^4] | Est. Cycles [^5] |
| ----------------------- | --------: | ----------: | ----------: | -----------: | ---------------: |
| decode_header           |        59 |          77 |           2 |            6 |              297 |
| decode_getresponse      |      5013 |        7006 |          15 |           94 |            10371 |
| decode_bool             |       119 |         163 |           3 |           13 |              633 |
| decode_counter32        |       150 |         197 |           2 |           14 |              697 |
| decode_counter64        |       150 |         196 |           3 |           14 |              701 |
| decode_gauge32          |       150 |         197 |           3 |           13 |              667 |
| decode_int              |       160 |         208 |           2 |           14 |              708 |
| decode_ipaddress        |       127 |         176 |           3 |           12 |              611 |
| decode_null             |       109 |         148 |           3 |           11 |              548 |
| decode_objectdescriptor |       114 |         161 |           2 |           11 |              556 |
| decode_oid              |       474 |         649 |           3 |           20 |             1364 |
| decode_octetstring      |       132 |         183 |           4 |           10 |              553 |
| decode_opaque           |       132 |         184 |           3 |           10 |              549 |
| decode_real_nr1         |       301 |         378 |           5 |           20 |             1103 |
| decode_real_nr2         |       480 |         593 |           5 |           42 |             2088 |
| decode_real_nr3         |       456 |         567 |           6 |           41 |             2032 |
| decode_relative_oid     |       384 |         529 |           3 |           18 |             1174 |
| decode_timeticks        |       166 |         218 |           4 |           11 |              623 |
| decode_uinteger32       |       166 |         217 |           2 |           14 |              717 |

[^1]: CPU instructions performed.
[^2]: L1 cache accesses.
[^3]: L2 cache accesses.
[^4]: RAM Accesses.
[^5]: Estimated CPU cycles.
[Iai]: https://github.com/bheisler/iai
[Rust]: https://www.rust-lang.org
[Cachegrind]: https://valgrind.org/docs/manual/cg-manual.html