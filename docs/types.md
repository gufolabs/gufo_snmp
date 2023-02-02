# Supported BER Types

*Gufo SNMP* implements minimalistic [X.690 BER][BER] encoder/decoder. It focuses only on types
and convenctions really used in SNMP protocol.

The currently supported types are:

| Type              | Class       | P/C[^1] |  Tag | Python Type          | Reference                      |
| ----------------- | ----------- | ------: | ---: | -------------------- | ------------------------------ |
| BOOLEAN           | Universal   |       P |    1 | bool                 | [X.690][X-690] pp 8.1          |
| INTEGER           | Universal   |       P |    2 | int                  | [X.690][X-690] pp 8.2          |
| BITSTRING         | Universal   |     P/C |    3 | :material-close:     | [X.690][X-690] pp 8.6          |
| OCTETSTRING       | Universal   |       P |    4 | bytes                | [X.690][X-690] pp 8.7          |
| NULL              | Universal   |       P |    5 | :material-close:     | [X.690][X-690] pp 8.8          |
| OBJECT IDENTIFIER | Universal   |       P |    6 | str                  | [X.690][X-690] pp 8.19         |
| OBJECT DESCRIPTOR | Universal   |     P/C |    7 | bytes                |                                |
| EXTERNAL          | Universal   |       P |    8 | :material-close:     | [X.690][X-690] pp 8.18         |
| REAL              | Universal   |       P |    9 | float                | [X.690][X-690] pp 8.5          |
| ENUMERATED        | Universal   |       P |   10 | :material-close:     |                                |
| RELATIVE OID      | Universal   |       P |   13 | str                  | [X.690][X-690] pp 8.20         |
| SEQUENCE          | Universal   |       C |   16 | :material-check:[^2] | [X.690][X-690] pp 8.9          |
| IpAddress         | Application |       P |    0 | str                  | [RFC-1442][RFC-1442] pp 7.1.5  |
| Counter32         | Application |       P |    1 | int                  | [RFC-1442][RFC-1442] pp 7.1.6  |
| Gauge32           | Application |       P |    2 | int                  | [RFC-1442][RFC-1442] pp 7.1.7  |
| TimeTicks         | Application |       P |    3 | int                  | [RFC-1442][RFC-1442] pp 7.1.8  |
| Opaque            | Application |       P |    4 | bytes                | [RFC-1442][RFC-1442] pp 7.1.9  |
| NsapAddress       | Application |       P |    5 | :material-close:     | [RFC-1442][RFC-1442] pp 7.1.10 |
| Counter64         | Application |       P |    6 | int                  | [RFC-1442][RFC-1442] pp 7.1.11 |
| UInteger32        | Application |       P |    7 | int                  | [RFC-1442][RFC-1442] pp 7.1.12 |
| noSuchObject      | Context     |       P |    0 | :material-check:[^3] | [RFC-1905][RFC-1905] pp 3      |
| noSuchInstance    | Context     |       P |    1 | :material-check:[^3] | [RFC-1905][RFC-1905] pp 3      |
| endOfMibView      | Context     |       P |    2 | :material-check:[^2] | [RFC-1905][RFC-1905] pp 3      |

[^1]: Primitive/Constructed
[^2]: Handled internally, never exposed
[^3]: Handled internally, raises NoSuchInstance or ignored.
[X-690]: https://www.itu.int/rec/T-REC-X.690
[BER]: https://en.wikipedia.org/wiki/X.690#BER_encoding
[RFC-1442]: https://datatracker.ietf.org/doc/html/rfc1442
[RFC-1905]: https://datatracker.ietf.org/doc/html/rfc1905