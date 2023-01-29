// ------------------------------------------------------------------------
// Gufo Snmp: BER OBJECT IDENTIFIER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerEncoder, BerHeader, ToPython, TAG_OBJECT_ID};
use crate::buf::Buffer;
use crate::error::SnmpError;
use pyo3::types::PyString;
use pyo3::{Py, PyAny, Python};

// Object identifier type
// According RFC-1442 pp 7.1.3:
// Any instance of this type may have at most
// 128 sub-identifiers.  Further, each sub-identifier must not
// exceed the value 2^32-1 (4294967295 decimal).
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SnmpOid(pub(crate) Vec<u32>);

impl<'a> BerDecoder<'a> for SnmpOid {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_OBJECT_ID;

    // Implement X.690 pp 8.19: Encoding of an object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        // First two elements
        let mut v = Vec::<u32>::new();
        v.extend_from_slice(&[(i[0] / 40) as u32, (i[0] % 40) as u32]);
        // Rest of them
        let mut b = 0;
        for x in i[1..h.length].iter() {
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
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
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
        buf.push_ber_len(buf.len() - start_len)?;
        // Push tag
        buf.push_u8(TAG_OBJECT_ID as u8)?;
        Ok(())
    }
}

impl ToPython for &SnmpOid {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        let v = self
            .0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(".");
        let v = PyString::new(py, &v);
        Ok(v.into())
    }
}

impl SnmpOid {
    #[inline]
    fn push_subelement(buf: &mut Buffer, se: u32) -> Result<(), SnmpError> {
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
    pub(crate) fn contains(&self, oid: &SnmpOid) -> bool {
        let sl = self.0.len();
        if oid.0.len() < sl {
            return false;
        }
        self.0 == oid.0[..sl]
    }
}

impl From<Vec<u32>> for SnmpOid {
    fn from(value: Vec<u32>) -> Self {
        SnmpOid(value)
    }
}

impl TryFrom<&str> for SnmpOid {
    type Error = SnmpError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        let mut last: u32 = 0;
        for c in value.bytes() {
            if c == 46 {
                // "."
                vec.push(last);
                last = 0;
                continue;
            }
            if (c >= 48) || (c <= 57) {
                last = last * 10 + ((c - 48) as u32);
            } else {
                return Err(SnmpError::InvalidData);
            }
        }
        vec.push(last);
        Ok(SnmpOid(vec))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Err;
    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [1u32, 3, 6, 1, 2, 1, 1, 5, 0];
        let (tail, v) = SnmpOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
    #[test]
    fn test_encode() -> Result<(), Err<SnmpError>> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::from(vec![1, 3, 6, 999, 3]);
        let expected = [6u8, 5, 0x2b, 0x06, 0x87, 0x67, 0x3];
        oid.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
    #[test]
    fn test_encode_decode() -> Result<(), Err<SnmpError>> {
        let mut buf = Buffer::default();
        let oid = SnmpOid::from(vec![1, 3, 6, 999, 3]);
        oid.push_ber(&mut buf)?;
        let (_, oid2) = SnmpOid::from_ber(&buf.data())?;
        assert_eq!(oid, oid2);
        Ok(())
    }
    #[test]
    fn test_try_from_str() -> Result<(), Err<SnmpError>> {
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
        assert_eq!(oid1.contains(&oid2), false);
    }
    #[test]
    fn test_contains2() {
        let oid1 = SnmpOid::from(vec![1, 3, 6]);
        let oid2 = SnmpOid::from(vec![1, 2, 5]);
        assert_eq!(oid1.contains(&oid2), false);
    }
    #[test]
    fn test_contains3() {
        let oid1 = SnmpOid::from(vec![1, 3, 6]);
        let oid2 = SnmpOid::from(vec![1, 3, 6, 1, 5]);
        assert_eq!(oid1.contains(&oid2), true);
    }
}
