// ------------------------------------------------------------------------
// Gufo Snmp: BER BOOLEAN class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_BOOL};
use crate::error::SnmpError;

pub(crate) struct SnmpBool(bool);

impl<'a> BerDecoder<'a> for SnmpBool {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_BOOL;

    // Implement X.690 pp 8.2: Encoding of a boolean value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        if h.length != 1 {
            return Err(SnmpError::InvalidData);
        }
        Ok(SnmpBool(i[0] != 0))
    }
}

impl SnmpBool {
    pub(crate) fn as_bool(&self) -> bool {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_long() {
        let data = [1u8, 2, 0, 0];
        let r = SnmpBool::from_ber(&data);
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [[1u8, 1, 0], [1, 1, 1], [1, 1, 255]];
        let expected = [false, true, true];
        for i in 0..data.len() {
            let (tail, v) = SnmpBool::from_ber(&data[i])?;
            assert_eq!(v.as_bool(), expected[i]);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }
}
