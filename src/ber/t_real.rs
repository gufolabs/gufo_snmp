// ------------------------------------------------------------------------
// Gufo SNMP: BER REAL Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, ToPython, TAG_REAL};
use crate::error::SnmpError;
use core::str::from_utf8;
use pyo3::{IntoPy, Py, PyAny, Python};

pub struct SnmpReal(f64);

impl<'a> BerDecoder<'a> for SnmpReal {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_REAL;

    // Implement X.690 pp 8.5: Encoding of a real value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        // zero-sized is a zero
        // 8.5.2: If the real value is the value plus zero,
        // there shall be no contents octets in the encoding.
        if h.is_empty() {
            return Ok(SnmpReal(0.0));
        }
        // 8.5.6: Check encoding
        Ok(SnmpReal(match i[0] {
            f if f & 0x80 == 0x80 => {
                // 8.5.7: Binary encoding

                // Bits
                // 8 7 6 5 4 3 2 1
                // | | | | | | | |
                // | | | | | | +-+-> Format of the exponent
                // | | | | +-+-> Scaling Factor: 8.5.7.3
                // | | +-+-> Base: 8.5.7.2
                // | +-> Sign: 8.5.7.1
                // +-> 1

                // 8.5.7.4 Bits 2 to 1 of the first contents octet
                // shall encode the format of the exponent as follows:
                let ln = (f & 0x03) as usize + 2;
                let e = SnmpReal::parse_u32(&i[1..ln]) as i32;
                let mut v: f64 = SnmpReal::parse_u32(&i[ln..]).into();
                // 8.5.7.3: Bits 4 to 3 of the first contents octet shall
                // encode the value of the binary scaling factor F
                // as an unsigned binary integer.
                match (f & 0x0c) >> 2 {
                    1 => v *= 2.0,
                    2 => v *= 4.0,
                    3 => v *= 8.0,
                    _ => return Err(SnmpError::InvalidData),
                }
                // 8.5.7.2: Bits 6 to 5 of the first contents octets
                // shall encode the value of the base B' as follows:
                // Bits6to5 => Base
                // 00 => base 2
                // 01 => base 8
                // 10 => base 16
                // 11 => Reserved for further editions of this Recommendation | International Standard.
                let base: f64 = match f & 0x30 {
                    0 => 2.0,
                    0x10 => 8.0,
                    0x20 => 16.0,
                    _ => return Err(SnmpError::InvalidData),
                };
                v *= base.powi(e);
                // 8.5.7.1: Bit 7 of the first contents octets
                // shall be 1 if S is –1 and 0 otherwise.
                if f & 0x40 == 0x40 {
                    v = -v
                }
                v
            }
            f if f & 0xc0 == 0 => {
                // 8.5.8: Decimal encoding
                // When decimal encoding is used (bits 8 to 7 = 00),
                // all the contents octets following the first contents
                // octet form a field, as the term is used in ISO 6093,
                // of a length chosen by the sender, and encoded according to ISO 6093.
                // The choice of ISO 6093 number representation is specified
                // by bits 6 to 1 of the first contents octet as follows:
                match f & 0x3f {
                    // ISO 6093 NR1: i.e. 456
                    1 => {
                        let s = from_utf8(&i[1..]).map_err(|_| SnmpError::InvalidData)?;
                        let v = s.parse::<i32>().map_err(|_| SnmpError::InvalidData)?;
                        v.into()
                    }
                    // ISO 6093 NR2: i.e. 456.7
                    2 => {
                        let s = from_utf8(&i[1..]).map_err(|_| SnmpError::InvalidData)?;
                        s.parse::<f64>().map_err(|_| SnmpError::InvalidData)?
                    }
                    // ISO 6093 NR3: i.e. 4567e-1
                    3 => {
                        let s = from_utf8(&i[1..]).map_err(|_| SnmpError::InvalidData)?;
                        s.parse::<f64>().map_err(|_| SnmpError::InvalidData)?
                    }
                    _ => return Err(SnmpError::InvalidData),
                }
            }
            0b01000000 => f64::INFINITY,
            0b01000001 => f64::NEG_INFINITY,
            0b01000010 => f64::NAN,
            0b01000011 => -0.0,
            _ => return Err(SnmpError::InvalidData),
        }))
    }
}

impl SnmpReal {
    fn parse_u32(i: &[u8]) -> u32 {
        let mut v = 0u32;
        for &n in i.iter() {
            v = (v << 8) | (n as u32);
        }
        v
    }
}

impl ToPython for &SnmpReal {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        Ok(self.0.into_py(py))
    }
}

impl From<SnmpReal> for f64 {
    fn from(value: SnmpReal) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    const EPSILON: f64 = 1e-10;
    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [
            vec![9u8, 0],
            // Decimal encoding
            vec![9u8, 4, 0x01, 0x34, 0x35, 0x36],       // NR1
            vec![9u8, 5, 0x01, 0x2d, 0x34, 0x35, 0x36], // NR1
            vec![9u8, 6, 0x02, 0x34, 0x35, 0x36, 0x2e, 0x37], // NR2
            vec![9u8, 7, 0x02, 0x2d, 0x34, 0x35, 0x36, 0x2e, 0x37], // NR2
            vec![9u8, 8, 0x03, 0x34, 0x35, 0x36, 0x37, 0x65, 0x2d, 0x31], // NR3
            vec![9u8, 9, 0x03, 0x2d, 0x34, 0x35, 0x36, 0x37, 0x65, 0x2d, 0x31], // NR3
            vec![9u8, 5, 0x03, 0x31, 0x45, 0x2b, 0x30], // NR3
            vec![9u8, 6, 0x03, 0x31, 0x35, 0x45, 0x2d, 0x31], // NR3
        ];
        let expected = [0.0, 456.0, -456.0, 456.7, -456.7, 456.7, -456.7, 1.0, 1.5];
        for i in 0..data.len() {
            let (tail, v) = SnmpReal::from_ber(&data[i])?;
            let diff = (v.0 - expected[i]).abs();
            assert!(diff < EPSILON);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }
    #[test]
    fn test_inf() -> Result<(), Err<SnmpError>> {
        let data = [9u8, 1, 0x40];
        let (tail, v) = SnmpReal::from_ber(&data)?;
        assert_eq!(v.0, f64::INFINITY);
        assert_eq!(tail.len(), 0);
        Ok(())
    }
    #[test]
    fn test_neg_inf() -> Result<(), Err<SnmpError>> {
        let data = [9u8, 1, 0x41];
        let (tail, v) = SnmpReal::from_ber(&data)?;
        assert_eq!(v.0, f64::NEG_INFINITY);
        assert_eq!(tail.len(), 0);
        Ok(())
    }
    #[test]
    fn test_nan() -> Result<(), Err<SnmpError>> {
        let data = [9u8, 1, 0x42];
        let (tail, v) = SnmpReal::from_ber(&data)?;
        assert!(v.0.is_nan());
        assert_eq!(tail.len(), 0);
        Ok(())
    }
    #[test]
    fn test_minus_zero() -> Result<(), Err<SnmpError>> {
        let data = [9u8, 1, 0x43];
        let (tail, v) = SnmpReal::from_ber(&data)?;
        assert_eq!(v.0, -0.0);
        assert!(v.0.is_sign_negative());
        assert_eq!(tail.len(), 0);
        Ok(())
    }
}
