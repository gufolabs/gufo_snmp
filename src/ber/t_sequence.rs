// ------------------------------------------------------------------------
// Gufo SNMP: BER SEQUENCE Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, TAG_SEQUENCE};
use crate::error::SnmpError;

pub struct SnmpSequence<'a>(pub(crate) &'a [u8]);

impl<'a> BerDecoder<'a> for SnmpSequence<'a> {
    const ALLOW_PRIMITIVE: bool = false;
    const ALLOW_CONSTRUCTED: bool = true;
    const TAG: Tag = TAG_SEQUENCE;

    // Implement X.690 pp 8.9: Encoding of a sequence value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        Ok(SnmpSequence(&i[..h.length]))
    }
}
