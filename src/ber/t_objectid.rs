// ------------------------------------------------------------------------
// Gufo Snmp: BER OBJECT IDENTIFIER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_OBJECT_ID};
use crate::error::SnmpError;

#[derive(Debug, PartialEq)]
pub(crate) struct SnmpOid(pub(crate) Vec<u64>);

impl<'a> BerDecoder<'a> for SnmpOid {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_OBJECT_ID;

    // Implement X.690 pp 8.19: Encoding of an object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        // First two elements
        let mut v = Vec::<u64>::new();
        v.extend_from_slice(&[(i[0] / 40) as u64, (i[0] % 40) as u64]);
        // Rest of them
        let mut b = 0;
        for x in i[1..h.length].iter() {
            b = (b << 7) + ((x & 0x7f) as u64);
            if x & 0x80 == 0 {
                v.push(b);
                b = 0;
            }
        }
        Ok(SnmpOid(v))
    }
}

impl From<Vec<u64>> for SnmpOid {
    fn from(value: Vec<u64>) -> Self {
        SnmpOid(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Err;
    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [1u64, 3, 6, 1, 2, 1, 1, 5, 0];
        let (tail, v) = SnmpOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
}
