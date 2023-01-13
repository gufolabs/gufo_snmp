// ------------------------------------------------------------------------
// Gufo Snmp: BER INTEGER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerEncoder, BerHeader, TAG_INT};
use crate::buf::Buffer;
use crate::error::SnmpError;
use std::cmp::Ordering;

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

const ZERO_BER: [u8; 3] = [2u8, 1, 0];

impl BerEncoder for SnmpInt {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
        match self.0.cmp(&0) {
            Ordering::Equal => {
                buf.push(&ZERO_BER)?;
                Ok(())
            }
            Ordering::Greater => {
                let start = buf.len();
                // Write body
                let mut left = self.0;
                while left > 0 {
                    buf.push_u8((left & 0xff) as u8)?;
                    left >>= 8;
                }
                // Write length
                buf.push_ber_len(buf.len() - start)?;
                // Write tag
                buf.push_u8(TAG_INT as u8)?;
                Ok(())
            }
            Ordering::Less => Err(SnmpError::NotImplemented),
        }
    }
}

impl From<i64> for SnmpInt {
    fn from(value: i64) -> Self {
        SnmpInt(value)
    }
}

impl From<SnmpInt> for u8 {
    fn from(value: SnmpInt) -> Self {
        value.0 as u8
    }
}

impl From<SnmpInt> for i64 {
    fn from(value: SnmpInt) -> Self {
        value.0
    }
}

impl SnmpInt {
    pub(crate) fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [
            vec![2u8, 1, 0],
            vec![2, 0],
            vec![2, 1, 0x7f],
            vec![2, 2, 0, 0x80],
            vec![2, 2, 1, 0],
            vec![2, 1, 0x80],
            vec![2, 1, 0xff, 0x7f],
        ];
        let expected = [0i64, 0, 127, 128, 256, -1, -128, -129];
        for i in 0..data.len() {
            let (tail, v) = SnmpInt::from_ber(&data[i])?;
            assert_eq!(v.0, expected[i]);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }

    #[test]
    fn test_encode() -> Result<(), SnmpError> {
        let mut buf = Buffer::default();
        let data = [0i64, 10, 500];
        let expected = [
            vec![2u8, 1, 0],
            vec![2, 1, 0x7f],
            vec![2, 2, 0, 0x80],
            vec![2, 2, 1, 0],
            vec![2, 1, 0x80],
            vec![2, 1, 0xff, 0x7f],
        ];
        for i in 0..data.len() {
            let si: SnmpInt = data[i].into();
            buf.reset();
            si.push_ber(&mut buf)?;
            assert_eq!(buf.data(), &expected[i]);
        }
        Ok(())
    }
}
