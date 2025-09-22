// ------------------------------------------------------------------------
// Gufo SNMP: GetBulk PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{BerDecoder, BerEncoder, SnmpInt, SnmpNull, SnmpOid, SnmpSequence};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use nom::IResult;

pub struct SnmpGetBulk<'a> {
    pub(crate) request_id: i64,
    pub(crate) non_repeaters: i64,
    pub(crate) max_repetitions: i64,
    pub(crate) vars: Vec<SnmpOid<'a>>,
}

impl<'a> TryFrom<&'a [u8]> for SnmpGetBulk<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // Request id
        let (tail, request_id) = SnmpInt::from_ber(value)?;
        // non-repeaters
        let (tail, non_repeaters) = SnmpInt::from_ber(tail)?;
        // max-repetitions
        let (tail, max_repetitions) = SnmpInt::from_ber(tail)?;
        // varbinds
        let (tail, vb) = SnmpSequence::from_ber(tail)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        let mut v_tail = vb.0;
        let mut vars = Vec::<SnmpOid>::new();
        while !v_tail.is_empty() {
            let (rest, oid) = SnmpGetBulk::parse_var(v_tail)?;
            vars.push(oid);
            v_tail = rest;
        }
        Ok(SnmpGetBulk {
            request_id: request_id.into(),
            non_repeaters: non_repeaters.into(),
            max_repetitions: max_repetitions.into(),
            vars,
        })
    }
}

impl BerEncoder for SnmpGetBulk<'_> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        // Push all vars in the reversed order
        let rest = buf.len();
        let null = SnmpNull {};
        for oid in self.vars.iter().rev() {
            let start = buf.len();
            // Trailing null
            null.push_ber(buf)?;
            // OID
            oid.push_ber(buf)?;
            // Enclosing sequence
            buf.push_tag_len(0x30, buf.len() - start)?;
        }
        // Enclosing sequence for varbinds
        // Spans for the end
        buf.push_tag_len(0x30, buf.len() - rest)?;
        // max-repetitions
        let max_repetitions: SnmpInt = self.max_repetitions.into();
        max_repetitions.push_ber(buf)?;
        // non-repeeaters
        let non_repeaters: SnmpInt = self.non_repeaters.into();
        non_repeaters.push_ber(buf)?;
        // Request id
        let r_id: SnmpInt = self.request_id.into();
        r_id.push_ber(buf)?;
        Ok(())
    }
}

impl SnmpGetBulk<'_> {
    fn parse_var(i: &[u8]) -> IResult<&[u8], SnmpOid<'_>, SnmpError> {
        // Parse enclosing sequence
        let (rest, vs) = SnmpSequence::from_ber(i)?;
        // Parse oid
        let (tail, oid) = SnmpOid::from_ber(vs.0)?;
        // Parse null
        let (_, _) = SnmpNull::from_ber(tail)?;
        Ok((rest, oid))
    }
}
