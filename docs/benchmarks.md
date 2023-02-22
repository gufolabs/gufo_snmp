# Benchmarks

## Rust Core Benchmarks

Gufo SNMP uses [Iai][Iai], a [Rust][Rust] [Cachegrind][Cachegrind]-based
bencmark framework to estimate performance of the critical code paths.

| Name                    | Inst.[^1] | L1 Acc.[^2] | L2 Acc.[^3] | RAM Acc.[^4] | Est. Cycles [^5] |
| ----------------------- | --------: | ----------: | ----------: | -----------: | ---------------: |
| buf_default             |       135 |         142 |           3 |            4 |              297 |
| buf_push_u8             |         5 |           6 |           1 |            1 |               46 |
| buf_push                |        58 |          79 |           2 |            4 |              229 |
| buf_push_ber_len_short  |        20 |          25 |           1 |            4 |              170 |
| buf_push_ber_len_long   |        22 |          30 |           0 |            3 |              130 |
| encode_get              |      2945 |        3917 |           8 |           76 |             6617 |
| decode_header           |        59 |          78 |           0 |            7 |              323 |
| decode_getresponse      |      4963 |        6953 |          13 |           93 |            10273 |
| decode_bool             |       119 |         163 |           3 |           13 |              633 |
| decode_counter32        |       143 |         190 |           1 |           13 |              650 |
| decode_counter64        |       143 |         188 |           2 |           14 |              688 |
| decode_gauge32          |       143 |         189 |           2 |           13 |              654 |
| decode_int              |       152 |         200 |           1 |           15 |              730 |
| decode_ipaddress        |       127 |         176 |           2 |           13 |              641 |
| decode_null             |       109 |         148 |           2 |           12 |              578 |
| decode_objectdescriptor |       114 |         160 |           1 |           13 |              620 |
| decode_oid              |       474 |         649 |           2 |           21 |             1394 |
| decode_octetstring      |       132 |         184 |           1 |           12 |              609 |
| decode_opaque           |       132 |         183 |           2 |           12 |              613 |
| decode_real_nr1         |       301 |         376 |           5 |           22 |             1171 |
| decode_real_nr2         |       480 |         594 |           5 |           41 |             2054 |
| decode_real_nr3         |       456 |         570 |           5 |           39 |             1960 |
| decode_relative_oid     |       384 |         529 |           1 |           20 |             1234 |
| decode_timeticks        |       159 |         211 |           2 |           11 |              606 |
| decode_uinteger32       |       159 |         209 |           2 |           13 |              674 |

[^1]: CPU instructions performed.
[^2]: L1 cache accesses.
[^3]: L2 cache accesses.
[^4]: RAM Accesses.
[^5]: Estimated CPU cycles.
[Iai]: https://github.com/bheisler/iai
[Rust]: https://www.rust-lang.org
[Cachegrind]: https://valgrind.org/docs/manual/cg-manual.html