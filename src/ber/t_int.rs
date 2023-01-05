// ------------------------------------------------------------------------
// Gufo Snmp: BER INTEGER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_INT};
use crate::error::SnmpError;

pub(crate) struct SnmpInt(i64);

impl<'a> BerDecoder<'a> for SnmpInt {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_INT;

    // Implement X.690 pp 8.3: Encoding of an integer value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        let mut v = 0i64;
        for n in i[..h.length].iter() {
            v = (v << 8) | (*n as i64);
        }
        if i[0] & 0x80 == 0x80 {
            // Negative number
            let m = 1 << (8 * h.length);
            v -= m;
        }
        Ok(SnmpInt(v))
    }
}

impl SnmpInt {
    pub(crate) fn as_i64(&self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [[2u8, 1, 0], [2u8, 1, 10]];
        let expected = [0i64, 10];
        for i in 0..data.len() {
            let (tail, v) = SnmpInt::from_ber(&data[i])?;
            assert_eq!(v.0, expected[i]);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }
}
