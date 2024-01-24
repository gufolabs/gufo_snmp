// ------------------------------------------------------------------------
// Gufo SNMP: BER RELATIVEE OBJECT IDENTIFIER Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, SnmpOid, Tag, TAG_RELATIVE_OID};
use crate::error::SnmpResult;

#[derive(Debug, PartialEq, Clone)]
pub struct SnmpRelativeOid(pub(crate) Vec<u32>);

impl<'a> BerDecoder<'a> for SnmpRelativeOid {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_RELATIVE_OID;

    // Implement X.690 pp 8.20: Encoding of a relative object identifier value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        let mut v = Vec::<u32>::with_capacity(h.length + 1);
        let mut b = 0;
        for &x in i[..h.length].iter() {
            b = (b << 7) + ((x & 0x7f) as u32);
            if x & 0x80 == 0 {
                v.push(b);
                b = 0;
            }
        }
        Ok(SnmpRelativeOid(v))
    }
}

impl From<Vec<u32>> for SnmpRelativeOid {
    fn from(value: Vec<u32>) -> Self {
        SnmpRelativeOid(value)
    }
}

impl SnmpRelativeOid {
    /// Apply relative oid to absolute one
    /// and return normalized absolute oid
    pub fn normalize(&self, oid: &SnmpOid) -> SnmpOid {
        let la = oid.0.len();
        let lr = self.0.len();
        if lr >= la {
            SnmpOid::from(self.0.clone())
        } else {
            let r = [oid.0[..la - lr].to_vec(), self.0.clone()].concat();
            SnmpOid(r)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Err;
    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [0xd, 4, 0xc2, 0x7b, 0x03, 0x02];
        let expected = [8571, 3, 2];
        let (tail, v) = SnmpRelativeOid::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(v.0, &expected);
        Ok(())
    }
    #[test]
    fn test_apply1() {
        let oid = SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 2, 2, 1, 10, 11]);
        let rel = SnmpRelativeOid::from(vec![12]);
        let expected = vec![1, 3, 6, 1, 2, 1, 2, 2, 1, 10, 12];
        let norm = rel.normalize(&oid);
        assert_eq!(norm.0, expected);
    }
    #[test]
    fn test_apply2() {
        let oid = SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 2, 2, 1, 10, 11]);
        let rel = SnmpRelativeOid::from(vec![11, 10]);
        let expected = vec![1, 3, 6, 1, 2, 1, 2, 2, 1, 11, 10];
        let norm = rel.normalize(&oid);
        assert_eq!(norm.0, expected);
    }
    #[test]
    fn test_apply_all1() {
        let oid = SnmpOid::from(vec![1, 3, 6, 1, 2]);
        let rel = SnmpRelativeOid::from(vec![1, 3, 6, 2, 1]);
        let expected = vec![1, 3, 6, 2, 1];
        let norm = rel.normalize(&oid);
        assert_eq!(norm.0, expected);
    }
    #[test]
    fn test_apply_all2() {
        let oid = SnmpOid::from(vec![1, 3, 6, 1, 2]);
        let rel = SnmpRelativeOid::from(vec![1, 3, 6, 2, 1, 5]);
        let expected = vec![1, 3, 6, 2, 1, 5];
        let norm = rel.normalize(&oid);
        assert_eq!(norm.0, expected);
    }
}
