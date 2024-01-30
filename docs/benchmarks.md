---
hide:
    - navigation
---
# Benchmarks

## Rust Core Benchmarks

Gufo SNMP uses [Iai][Iai], a [Rust][Rust] [Cachegrind][Cachegrind]-based
bencmark framework to estimate performance of the critical code paths.

| Name                    | Inst.[^1] | L1 Acc.[^2] | L2 Acc.[^3] | RAM Acc.[^4] | Est. Cycles [^5] |
| ----------------------- | --------: | ----------: | ----------: | -----------: | ---------------: |
| md5_default             |        36 |          40 |           2 |            0 |               50 |
| md5_password_to_master  |  14923437 |    17486016 |           4 |           55 |         17487961 |
| md5_localize            |       939 |        1092 |           4 |           52 |             2932 |
| sha1_default            |        36 |          40 |           2 |            0 |               50 |
| sha1_password_to_master |  26671569 |    31970424 |           3 |          101 |         31973974 |
| sha1_localize           |      1747 |        2051 |           3 |           97 |             5461 |
| buf_default             |         0 |           0 |           0 |            2 |               40 |
| buf_push_u8             |         2 |           3 |           1 |            0 |                8 |
| buf_push                |       113 |         121 |           6 |           21 |              886 |
| buf_push_ber_len_short  |        92 |          95 |           5 |           19 |              785 |
| buf_push_ber_len_long   |       111 |         117 |           4 |           20 |              837 |
| decode_header           |        90 |         114 |           1 |            5 |              294 |
| decode_getresponse      |      4967 |        6922 |           8 |           91 |            10147 |
| decode_bool             |       120 |         153 |           2 |            9 |              478 |
| decode_counter32        |       154 |         188 |           2 |           11 |              583 |
| decode_counter64        |       155 |         191 |           1 |           10 |              546 |
| decode_gauge32          |       154 |         187 |           3 |           11 |              587 |
| decode_int              |       165 |         200 |           2 |           12 |              630 |
| decode_ipaddress        |       130 |         169 |           1 |           10 |              524 |
| decode_null             |       102 |         127 |           1 |            7 |              377 |
| decode_objectdescriptor |       116 |         149 |           3 |            9 |              479 |
| decode_oid              |       440 |         598 |           1 |           14 |             1093 |
| decode_octetstring      |        98 |         134 |           1 |            8 |              419 |
| decode_opaque           |        98 |         131 |           3 |            9 |              461 |
| decode_real_nr1         |       289 |         361 |           5 |           21 |             1121 |
| decode_real_nr2         |       446 |         548 |           5 |           40 |             1973 |
| decode_real_nr3         |       429 |         525 |           5 |           40 |             1950 |
| decode_relative_oid     |       347 |         471 |           2 |           15 |             1006 |
| decode_timeticks        |       135 |         169 |           2 |           11 |              564 |
| decode_uinteger32       |       135 |         169 |           2 |           11 |              564 |
| encode_get              |      2586 |        3405 |          10 |           84 |             6395 |

[^1]: CPU instructions performed.
[^2]: L1 cache accesses.
[^3]: L2 cache accesses.
[^4]: RAM Accesses.
[^5]: Estimated CPU cycles.
[Iai]: https://github.com/bheisler/iai
[Rust]: https://www.rust-lang.org
[Cachegrind]: https://valgrind.org/docs/manual/cg-manual.html