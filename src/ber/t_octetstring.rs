// ------------------------------------------------------------------------
// Gufo Snmp
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_OCTET_STRING};
use crate::error::SnmpError;

pub(crate) struct SnmpOctetString<'a>(pub(crate) &'a [u8]);

impl<'a> BerDecoder<'a> for SnmpOctetString<'a> {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_OCTET_STRING;

    // Implement X.690 pp 8.7: Encoding of an octetstring value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        Ok(SnmpOctetString(&i[..h.length]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [4u8, 5, 0, 1, 2, 3, 4];
        let (tail, s) = SnmpOctetString::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(s.0, &data[2..]);
        Ok(())
    }
}
