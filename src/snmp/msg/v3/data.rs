// ------------------------------------------------------------------------
// Gufo SNMP: MsgData
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::scoped::ScopedPdu;
use crate::ber::{BerDecoder, BerEncoder, SnmpOctetString, TAG_OCTET_STRING};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};

pub enum MsgData<'a> {
    Plaintext(ScopedPdu<'a>),
    Encrypted(&'a [u8]),
}

impl<'a> TryFrom<&'a [u8]> for MsgData<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> SnmpResult<MsgData<'a>> {
        if i.is_empty() {
            return Err(SnmpError::Incomplete);
        }
        Ok(if i[0] == TAG_OCTET_STRING {
            // Encryped
            let (_, os) = SnmpOctetString::from_ber(i)?;
            MsgData::Encrypted(os.0)
        } else {
            // Plaintext
            MsgData::Plaintext(ScopedPdu::try_from(i)?)
        })
    }
}

impl BerEncoder for MsgData<'_> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        match self {
            MsgData::Plaintext(x) => x.push_ber(buf),
            MsgData::Encrypted(x) => buf.push_tagged(TAG_OCTET_STRING, x),
        }
    }
}
