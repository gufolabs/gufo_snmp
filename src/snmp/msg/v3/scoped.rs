// ------------------------------------------------------------------------
// Gufo SNMP: Scoped PDU
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{BerDecoder, BerEncoder, SnmpOctetString, SnmpSequence, TAG_OCTET_STRING};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::pdu::SnmpPdu;

pub struct ScopedPdu<'a> {
    pub engine_id: &'a [u8],
    pub pdu: SnmpPdu<'a>,
}

const EMPTY_BER: [u8; 2] = [TAG_OCTET_STRING, 0];

impl<'a> TryFrom<&'a [u8]> for ScopedPdu<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> SnmpResult<ScopedPdu<'a>> {
        let (_, envelope) = SnmpSequence::from_ber(i)?;
        // Context engine id
        let (tail, engine_id) = SnmpOctetString::from_ber(envelope.0)?;
        // Context engine name
        let (tail, _ctx_engine_name) = SnmpOctetString::from_ber(tail)?;
        // Decode PDU and return
        Ok(ScopedPdu {
            engine_id: engine_id.0,
            pdu: SnmpPdu::try_from(tail)?,
        })
    }
}

impl<'a> BerEncoder for ScopedPdu<'a> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        let rest = buf.len();
        // Push PDU
        self.pdu.push_ber(buf)?;
        // Push context engine name
        buf.push(&EMPTY_BER)?;
        // Push context engine id
        let ln = self.engine_id.len();
        if ln > 0 {
            buf.push_tagged(TAG_OCTET_STRING, self.engine_id)?;
        } else {
            buf.push(&EMPTY_BER)?;
        }
        // Push option header
        buf.push_ber_len(buf.len() - rest)?;
        buf.push_u8(0x30)?;
        Ok(())
    }
}
