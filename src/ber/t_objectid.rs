// ------------------------------------------------------------------------
// Gufo SNMP: BER OBJECT IDENTIFIER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerEncoder, BerHeader, TAG_OBJECT_ID, Tag};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use pyo3::types::PyString;
use pyo3::{Bound, IntoPyObject, PyAny, Python};
use std::fmt::Write;

// Object identifier type
// According RFC-1442 pp 7.1.3:
// Any instance of this type may have at most
// 128 sub-identifiers.  Further, each sub-identifier must not
// exceed the value 2^32-1 (4294967295 decimal).
// We store raw BER encoding.
// We use owned implementation to process relative oids correctly.
#[derive(Debug, PartialEq, Clone)]
pub struct SnmpOid(pub(crate) Vec<u8>);

impl<'a> BerDecoder<'a> for SnmpOid {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_OBJECT_ID;

    // Implement X.690 pp 8.19: Encoding of an object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        Ok(SnmpOid(Vec::from(&i[..h.length])))
    }
}

impl BerEncoder for SnmpOid {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        buf.push(&self.0)?;
        buf.push_tag_len(TAG_OBJECT_ID, self.0.len())
    }
}

impl<'py> IntoPyObject<'py> for &SnmpOid {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let r = String::try_from(self)?;
        Ok(PyString::new(py, &r).into_any())
    }
}

impl TryFrom<&SnmpOid> for String {
    type Error = SnmpError;

    fn try_from(value: &SnmpOid) -> Result<Self, Self::Error> {
        let mut r = String::with_capacity(value.0.len() * 5);
        let mut iter = value.0.iter();
        // First two subelements
        let first = iter.next().ok_or(SnmpError::InvalidData)?;
        write!(r, "{}.{}", first / 40, first % 40).map_err(|_| SnmpError::InvalidData)?;
        let mut b = 0u32;
        for c in iter {
            b = (b << 7) + ((*c as u32) & 0x7f);
            if c & 0x80 == 0 {
                write!(r, ".{}", b).map_err(|_| SnmpError::InvalidData)?;
                b = 0;
            }
        }
        Ok(r)
    }
}

impl SnmpOid {
    // Check oid is contained within
    #[inline]
    pub fn starts_with(&self, oid: &SnmpOid) -> bool {
        oid.0.starts_with(&self.0)
    }
}

struct OidSubelementIterator<'a>(core::str::Split<'a, &'a str>);

impl<'a> OidSubelementIterator<'a> {
    pub fn new(oid: &'a str) -> Self {
        Self(oid.split("."))
    }
}

impl Iterator for OidSubelementIterator<'_> {
    type Item = Result<u32, SnmpError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|part| part.parse::<u32>().map_err(|_| SnmpError::InvalidData))
    }
}

impl TryFrom<&str> for SnmpOid {
    type Error = SnmpError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // `1.3.` encoded as one octet.
        // For other subelements the BER representation
        // usually shorter that string one.
        // So  `value.len() - 3` is a good estimate.
        // As this conversion mostly used for input parameters
        // excessive memory usage is not significant.
        let mut vec = Vec::<u8>::with_capacity(value.len().saturating_sub(3));
        // Subelements iterator
        let mut iter = OidSubelementIterator::new(value);
        // Get first two subelements
        let first = iter.next().ok_or(SnmpError::InvalidData)??.min(6) as u8;
        let second = iter.next().ok_or(SnmpError::InvalidData)??.min(39) as u8;
        vec.push(40 * first + second);
        // Push other elements
        for sr in iter {
            let sub_id = sr?;
            if sub_id <= 0x7F {
                vec.push(sub_id as u8);
            } else if sub_id <= 0x3FFF {
                vec.push(((sub_id >> 7) as u8) | 0x80);
                vec.push((sub_id as u8) & 0x7F);
            } else if sub_id <= 0x1F_FFFF {
                vec.push(((sub_id >> 14) as u8) | 0x80);
                vec.push((((sub_id >> 7) as u8) & 0x7F) | 0x80);
                vec.push((sub_id as u8) & 0x7F);
            } else if sub_id <= 0x0FFF_FFFF {
                vec.push(((sub_id >> 21) as u8) | 0x80);
                vec.push((((sub_id >> 14) as u8) & 0x7F) | 0x80);
                vec.push((((sub_id >> 7) as u8) & 0x7F) | 0x80);
                vec.push((sub_id as u8) & 0x7F);
            } else {
                vec.push(((sub_id >> 28) as u8) | 0x80);
                vec.push((((sub_id >> 21) as u8) & 0x7F) | 0x80);
                vec.push((((sub_id >> 14) as u8) & 0x7F) | 0x80);
                vec.push((((sub_id >> 7) as u8) & 0x7F) | 0x80);
                vec.push((sub_id as u8) & 0x7F);
            }
        }
        // Done
        Ok(SnmpOid(vec))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [43, 6, 1, 2, 1, 1, 5, 0];
        let (tail, v) = SnmpOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
    #[test]
    fn test_encode() -> SnmpResult<()> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::try_from("1.3.6.999.3")?;
        let expected = [6u8, 5, 0x2b, 0x06, 0x87, 0x67, 0x3];
        oid.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
    #[test]
    fn test_encode_decode() -> SnmpResult<()> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::try_from("1.3.6.999.3")?;
        oid.push_ber(&mut buf)?;
        let (_, oid2) = SnmpOid::from_ber(buf.data())?;
        assert_eq!(oid, oid2);
        Ok(())
    }

    #[test_case("1.3.6.999.3", vec![43, 6, 135, 103, 3]; "1")]
    #[test_case("1.3.6.1.2.1.1.5.0", vec![43, 6, 1, 2, 1, 1, 5, 0]; "2")]
    fn test_try_from_str(data: &str, expected: Vec<u8>) -> SnmpResult<()> {
        let oid = SnmpOid::try_from(data)?;
        assert_eq!(oid.0, expected);
        Ok(())
    }

    #[test_case("1.3.6", "1.3", false; "1")]
    #[test_case("1.3.6", "1.2.5", false; "2")]
    #[test_case("1.3.6", "1.3.6.1.5", true; "3")]
    fn test_starts_with(x: &str, y: &str, expected: bool) -> SnmpResult<()> {
        let oid1 = SnmpOid::try_from(x)?;
        let oid2 = SnmpOid::try_from(y)?;
        assert_eq!(oid1.starts_with(&oid2), expected);
        Ok(())
    }

    #[test_case(vec![43, 6], "1.3.6"; "1")]
    #[test_case(vec![43, 6, 135, 103, 3], "1.3.6.999.3"; "2")]
    #[test_case(vec![43, 6, 1, 2, 1, 1, 5, 0], "1.3.6.1.2.1.1.5.0"; "3")]
    fn test_to_string(data: Vec<u8>, expected: &str) -> SnmpResult<()> {
        let oid = SnmpOid(data);
        let s = String::try_from(&oid)?;
        assert_eq!(s, expected);
        Ok(())
    }
}
