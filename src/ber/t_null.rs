// ------------------------------------------------------------------------
// Gufo SNMP: BER NULL Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerEncoder, BerHeader, TAG_NULL, Tag};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};

pub struct SnmpNull;

impl<'a> BerDecoder<'a> for SnmpNull {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_NULL;

    // Implement X.690 pp 8.8: Encoding of a null value
    fn decode(_: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        if h.length != 0 {
            return Err(SnmpError::InvalidTagFormat);
        }
        Ok(SnmpNull)
    }
}

const NULL_BER: [u8; 2] = [5u8, 0];

impl BerEncoder for SnmpNull {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        buf.push(&NULL_BER)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> SnmpResult<()> {
        let data = [5u8, 0];
        let (tail, _) = SnmpNull::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        Ok(())
    }
    #[test]
    fn test_invalid_length() {
        let data = [5u8, 1, 0];
        let r = SnmpNull::from_ber(&data);
        assert!(r.is_err());
    }
    #[test]
    fn test_encode() -> SnmpResult<()> {
        let mut b = Buffer::default();
        SnmpNull {}.push_ber(&mut b)?;
        let expected = [5u8, 0];
        assert_eq!(b.data(), &expected);
        Ok(())
    }
    #[test]
    fn test_encode_decode() -> SnmpResult<()> {
        let mut b = Buffer::default();
        SnmpNull {}.push_ber(&mut b)?;
        SnmpNull::from_ber(b.data())?;
        Ok(())
    }
}
