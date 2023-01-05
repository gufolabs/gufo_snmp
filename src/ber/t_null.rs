// ------------------------------------------------------------------------
// Gufo Snmp: BER NULL Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_NULL};
use crate::error::SnmpError;

pub(crate) struct SnmpNull;

impl<'a> BerDecoder<'a> for SnmpNull {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_NULL;

    // Implement X.690 pp 8.8: Encoding of a null value
    fn decode(_: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        if h.length != 0 {
            return Err(SnmpError::InvalidTagFormat);
        }
        Ok(SnmpNull)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse() -> Result<(), Err<SnmpError>> {
        let data = [5u8, 0];
        let (tail, _) = SnmpNull::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        Ok(())
    }
    // @todo: test invalid length
}
