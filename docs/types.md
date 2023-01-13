# Supported BER types

| Type              | Class       | P/C[^1] |  Tag | Supported        | Python Type | Reference                      |
| ----------------- | ----------- | ------: | ---: | ---------------- | ----------- | ------------------------------ |
| BOOLEAN           | Universal   |       P |    1 | :material-check: |             | [X.690][X-690] pp 8.1          |
| INTEGER           | Universal   |       P |    2 | :material-check: |             | [X.690][X-690] pp 8.2          |
| BITSTRING         | Universal   |     P/C |    3 | :material-close: |             | [X.690][X-690] pp 8.6          |
| OCTETSTRING       | Universal   |       P |    4 | :material-check: | bytes       | [X.690][X-690] pp 8.7          |
| NULL              | Universal   |       P |    5 | :material-check: |             | [X.690][X-690] pp 8.8          |
| OBJECT IDENTIFIER | Universal   |       P |    6 | :material-check: | str         | [X.690][X-690] pp 8.19         |
| - incremental     |             |         |      | :material-close: |             |                                |
| OBJECT DESCRIPTOR | Universal   |     P/C |    7 | :material-close: |             |                                |
| EXTERNAL          | Universal   |       P |    8 | :material-close: |             | [X.690][X-690] pp 8.18         |
| REAL              | Universal   |       P |    9 | :material-close: |             | [X.690][X-690] pp 8.5          |
| ENUMERATED        | Universal   |       P |   10 | :material-close: |             |                                |
| SEQUENCE          | Universal   |       C |   16 | :material-check: |             | [X.690][X-690] pp 8.9          |
| IpAddress         | Application |       P |    0 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.5  |
| Counter32         | Application |       P |    1 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.6  |
| Gauge32           | Application |       P |    2 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.7  |
| TimeTicks         | Application |       P |    3 | :material-check: | int         | [RFC-1442][RFC-1442] pp 7.1.8  |
| Opaque            | Application |       P |    4 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.9  |
| NsapAddress       | Application |       P |    5 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.10 |
| Counter64         | Application |       P |    6 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.11 |
| UInteger32        | Application |       P |    7 | :material-close: |             | [RFC-1442][RFC-1442] pp 7.1.12 |
| noSuchObject      | Context     |       P |    0 | :material-close: |             | [RFC-1905][RFC-1095] pp 3      |
| noSuchInstance    | Context     |       P |    1 | :material-close: |             | [RFC-1905][RFC-1095] pp 3      |
| endOfMibView      | Context     |       P |    2 | :material-close: |             | [RFC-1905][RFC-1095] pp 3      |

[^1]: Primitive/Constructed
[X-690]: https://www.itu.int/rec/T-REC-X.690
[RFC-1442]: https://datatracker.ietf.org/doc/html/rfc1442
[RFC-1905]: https://datatracker.ietf.org/doc/html/rfc1905