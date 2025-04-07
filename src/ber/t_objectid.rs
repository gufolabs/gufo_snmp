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

// Object identifier type
// According RFC-1442 pp 7.1.3:
// Any instance of this type may have at most
// 128 sub-identifiers.  Further, each sub-identifier must not
// exceed the value 2^32-1 (4294967295 decimal).
#[derive(Debug, PartialEq, Clone)]
pub struct SnmpOid(pub(crate) Vec<u32>);

impl<'a> BerDecoder<'a> for SnmpOid {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_OBJECT_ID;

    // Implement X.690 pp 8.19: Encoding of an object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        // First two elements
        let mut v = Vec::<u32>::with_capacity(h.length + 1);
        let first = i[0] as u32;
        let (si1, si2) = (first / 40, first % 40);
        v.push(si1);
        v.push(si2);
        // Rest of them
        let mut b = 0;
        for &x in i[1..h.length].iter() {
            b = (b << 7) + ((x & 0x7f) as u32);
            if x & 0x80 == 0 {
                v.push(b);
                b = 0;
            }
        }
        Ok(SnmpOid(v))
    }
}

impl BerEncoder for SnmpOid {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        if self.0.len() < 2 {
            return Err(SnmpError::InvalidData); // Too short oid
        }
        let start_len = buf.len();
        for &se in self.0[2..].iter().rev() {
            SnmpOid::push_subelement(buf, se)?;
        }
        // First two elements are pushed in one octet
        SnmpOid::push_subelement(buf, self.0[0] * 40 + self.0[1])?;
        // Push length
        buf.push_tag_len(TAG_OBJECT_ID, buf.len() - start_len)
    }
}

impl<'py> IntoPyObject<'py> for &SnmpOid {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let v = self
            .0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(".");
        Ok(PyString::new(py, &v).into_any())
    }
}

impl SnmpOid {
    #[inline]
    fn push_subelement(buf: &mut Buffer, se: u32) -> SnmpResult<()> {
        let mut left = se;
        // Push least significant 7 bits
        buf.push_u8((left & 0x7f) as u8)?;
        left >>= 7;
        while left > 0 {
            //  Push next least significant 7 bits with highest-bit set
            buf.push_u8(((left & 0x7f) as u8) | 0x80)?;
            left >>= 7;
        }
        Ok(())
    }
    // Check oid is contained within
    pub fn starts_with(&self, oid: &SnmpOid) -> bool {
        oid.0.starts_with(&self.0)
    }
}

impl From<Vec<u32>> for SnmpOid {
    fn from(value: Vec<u32>) -> Self {
        SnmpOid(value)
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
        let vec = OidSubelementIterator::new(value).collect::<Result<Vec<u32>, Self::Error>>()?;
        Ok(SnmpOid(vec))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [1u32, 3, 6, 1, 2, 1, 1, 5, 0];
        let (tail, v) = SnmpOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
    #[test]
    fn test_encode() -> SnmpResult<()> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::from(vec![1, 3, 6, 999, 3]);
        let expected = [6u8, 5, 0x2b, 0x06, 0x87, 0x67, 0x3];
        oid.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
    #[test]
    fn test_encode_decode() -> SnmpResult<()> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::from(vec![1, 3, 6, 999, 3]);
        oid.push_ber(&mut buf)?;
        let (_, oid2) = SnmpOid::from_ber(buf.data())?;
        assert_eq!(oid, oid2);
        Ok(())
    }
    #[test]
    fn test_try_from_str() -> SnmpResult<()> {
        let data = ["1.3.6.999.3", "1.3.6.1.2.1.1.5.0"];
        let expected = vec![vec![1u32, 3, 6, 999, 3], vec![1u32, 3, 6, 1, 2, 1, 1, 5, 0]];
        for i in 0..data.len() {
            let s = SnmpOid::try_from(data[i])?;
            assert_eq!(s.0, expected[i]);
        }
        Ok(())
    }
    #[test]
    fn test_contains1() {
        let oid1 = SnmpOid::from(vec![1, 3, 6]);
        let oid2 = SnmpOid::from(vec![1, 3]);
        assert!(!oid1.starts_with(&oid2));
    }
    #[test]
    fn test_contains2() {
        let oid1 = SnmpOid::from(vec![1, 3, 6]);
        let oid2 = SnmpOid::from(vec![1, 2, 5]);
        assert!(!oid1.starts_with(&oid2));
    }
    #[test]
    fn test_contains3() {
        let oid1 = SnmpOid::from(vec![1, 3, 6]);
        let oid2 = SnmpOid::from(vec![1, 3, 6, 1, 5]);
        assert!(oid1.starts_with(&oid2));
    }
}
