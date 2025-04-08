// ------------------------------------------------------------------------
// Gufo SNMP: BER RELATIVEE OBJECT IDENTIFIER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, SnmpOid, TAG_RELATIVE_OID, Tag};
use crate::error::SnmpResult;

#[derive(Debug, PartialEq, Clone)]
pub struct SnmpRelativeOid(pub(crate) Vec<u8>);

impl<'a> BerDecoder<'a> for SnmpRelativeOid {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_RELATIVE_OID;

    // Implement X.690 pp 8.20: Encoding of a relative object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        Ok(SnmpRelativeOid(Vec::from(&i[..h.length])))
    }
}

impl SnmpRelativeOid {
    /// Apply relative oid to absolute one
    /// and return normalized absolute oid
    pub fn normalize(&self, oid: &SnmpOid) -> SnmpOid {
        // Number of subelements
        let rel_si = SnmpRelativeOid::subelements(&self.0);
        // Number of subelements in base. First octet holds 2 subidentifiers.
        let base_si = SnmpRelativeOid::subelements(&oid.0[1..]) + 2;
        //
        if rel_si < base_si - 2 {
            let offset = SnmpRelativeOid::find_subelement(&oid.0[1..], base_si - rel_si - 2)
                .unwrap_or(0)
                + 1;
            let mut r = Vec::with_capacity(oid.0.len() + self.0.len());
            r.extend_from_slice(&oid.0[..offset]);
            r.extend_from_slice(&self.0);
            SnmpOid(r)
        } else {
            // Replace fully
            // First value is collapsed to one
            let mut r = Vec::with_capacity(self.0.len() - 1);
            // Collapse first two values into one octet
            r.push(self.0[0] * 40 + self.0[1]);
            // Push others
            r.extend_from_slice(&self.0[2..]);
            SnmpOid(r)
        }
    }
    // Calculate number of subelements
    #[inline]
    fn subelements(data: &[u8]) -> usize {
        data.iter().filter(|&c| c & 0x80 == 0).count()
    }
    // Calculate subelement offset
    #[inline]
    fn find_subelement(data: &[u8], n: usize) -> Option<usize> {
        let mut left = n;
        let mut start = 0;
        for (offset, c) in data.iter().enumerate() {
            if left == 0 {
                return (start < data.len()).then_some(start);
            }
            if c & 0x80 == 0 {
                left -= 1;
                start = offset + 1;
            }
        }
        None
    }
}

// For tests
#[cfg(test)]
impl TryFrom<Vec<u8>> for SnmpRelativeOid {
    type Error = std::convert::Infallible;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![6, 1, 2, 1, 22, 1, 10, 11], 8; "Short elements")]
    #[test_case(vec![12], 1; "Single element")]
    #[test_case(vec![6, 1, 2, 135, 103, 1], 5; "Long element")]
    fn test_subelements(data: Vec<u8>, expected: usize) {
        assert_eq!(SnmpRelativeOid::subelements(&data), expected);
    }

    #[test_case(vec![12], 0, Some(0);"Zero element 1")]
    #[test_case(vec![12, 25], 0, Some(0);"Zero element 2")]
    #[test_case(vec![12, 25], 1, Some(1);"First element short")]
    #[test_case(vec![12, 135, 103], 1, Some(1);"First element long")]
    #[test_case(vec![12, 135, 103, 12, 135, 103, 12, 135, 103, 12], 5, Some(7);"Fifth element long")]
    fn test_find_subelement(data: Vec<u8>, n: usize, expected: Option<usize>) {
        let offset = SnmpRelativeOid::find_subelement(&data, n);
        assert_eq!(offset, expected);
    }

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [0xd, 4, 0xc2, 0x7b, 0x03, 0x02];
        let expected = [194, 123, 3, 2];
        let (tail, v) = SnmpRelativeOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
    #[test_case("1.3.6.1.2.1.2.2.1.10.11", vec![12], vec![43, 6, 1, 2, 1, 2, 2, 1, 10, 12]; "Single element")]
    #[test_case("1.3.6.1.2.1.2.2.1.10.11", vec![11,10], vec![43, 6, 1, 2, 1, 2, 2, 1, 11, 10]; "Two elements")]
    #[test_case("1.3.6.1.2", vec![1,3,6,1,2], vec![43,6,1,2]; "Replace same")]
    #[test_case("1.3.6.1.2", vec![1,3,6,2,1,5], vec![43,6,2,1,5]; "Replace all")]
    fn test_normalize(base: &str, relative: Vec<u8>, expected: Vec<u8>) -> SnmpResult<()> {
        let oid = SnmpOid::try_from(base)?;
        let rel = SnmpRelativeOid::try_from(relative)?;
        let norm = rel.normalize(&oid);
        assert_eq!(norm.0, expected);
        Ok(())
    }
}
