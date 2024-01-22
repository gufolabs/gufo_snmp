// ------------------------------------------------------------------------
// Gufo SNMP: BER INTEGER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerEncoder, BerHeader, Tag, ToPython, TAG_INT};
use crate::buf::Buffer;
use crate::error::{SnmpResult};
use pyo3::{IntoPy, Py, PyAny, Python};
use std::cmp::Ordering;

pub struct SnmpInt(i64);

impl<'a> BerDecoder<'a> for SnmpInt {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_INT;

    // Implement X.690 pp 8.3: Encoding of an integer value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        if h.is_empty() {
            return Ok(SnmpInt(0));
        }
        let v = i
            .iter()
            .take(h.length)
            .map(|x| *x as i64)
            .reduce(|acc, x| (acc << 8) | x)
            .unwrap_or(0);
        Ok(SnmpInt(if i[0] & 0x80 == 0 {
            v
        } else {
            // Negative number
            let m = 1 << (8 * h.length);
            v - m
        }))
    }
}

const ZERO_BER: [u8; 3] = [TAG_INT, 1, 0];

impl BerEncoder for SnmpInt {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        match self.0.cmp(&0) {
            Ordering::Equal => {
                buf.push(&ZERO_BER)?;
                Ok(())
            }
            Ordering::Greater => {
                let start = buf.len();
                // Write body
                let mut left = self.0;
                loop {
                    buf.push_u8((left & 0xff) as u8)?;
                    if left < 0xff {
                        if left & 0x80 == 0x80 {
                            // Highest bit is 1
                            // push leading zero
                            buf.push_u8(0)?;
                        }
                        break;
                    }
                    left >>= 8;
                }
                // Write length
                buf.push_ber_len(buf.len() - start)?;
                // Write tag
                buf.push_u8(TAG_INT)?;
                Ok(())
            }
            Ordering::Less => {
                let start = buf.len();
                let mut left = -self.0;
                // Calculate used octets
                let mut ln = 0;
                while left > 0 {
                    ln += 1;
                    left >>= 8;
                }
                // Calculate complement
                let d = 1 << (ln * 8 - 1);
                left = -self.0;
                let comp = if d < left { d << 8 } else { d };
                // Write octets
                if comp == left {
                    for _ in 0..ln - 1 {
                        buf.push_u8(0)?;
                    }
                    buf.push_u8(0x80)?;
                } else {
                    left = comp - left;
                    loop {
                        if left < 0xff {
                            buf.push_u8(0x80 | (left as u8))?;
                            break;
                        }
                        buf.push_u8((left & 0xff) as u8)?;
                        left >>= 8;
                    }
                }
                // Write length
                buf.push_ber_len(buf.len() - start)?;
                // Write tag
                buf.push_u8(TAG_INT)?;
                Ok(())
            }
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
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl ToPython for &SnmpInt {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>> {
        Ok(self.0.into_py(py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [
            vec![2u8, 1, 0],              // 0
            vec![2, 0],                   // 0
            vec![2, 1, 1],                // 1
            vec![2, 1, 0x7f],             // 127
            vec![2, 2, 0, 0x80],          // 128
            vec![2, 2, 1, 0],             // 256
            vec![2, 1, 0x80],             // -128
            vec![2, 2, 0xff, 0x7f],       // -129
            vec![2, 3, 0xff, 0, 1],       // -65535
            vec![2, 2, 0x20, 0x85],       // 0x2085
            vec![2, 3, 0x20, 0x85, 0x11], // 0x208511
        ];
        let expected = [
            0i64, 0, 1, 127, 128, 256, -128, -129, -65535, 0x2085, 0x208511,
        ];
        for i in 0..data.len() {
            let (tail, v) = SnmpInt::from_ber(&data[i])?;
            assert_eq!(v.0, expected[i]);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }

    #[test]
    fn test_encode() -> SnmpResult<()> {
        let mut buf = Buffer::default();
        let data = [0i64, 1, 127, 128, 256, -128, -129, -65535];
        let expected = [
            vec![2u8, 1, 0],        // 0
            vec![2, 1, 1],          // 1
            vec![2, 1, 0x7f],       // 127
            vec![2, 2, 0, 0x80],    // 128
            vec![2, 2, 1, 0],       // 256
            vec![2, 1, 0x80],       // -128
            vec![2, 2, 0xff, 0x7f], // -129
            vec![2, 3, 0xff, 0, 1], // -65535
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
