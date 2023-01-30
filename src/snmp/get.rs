// ------------------------------------------------------------------------
// Gufo SNMP: GET PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{BerDecoder, BerEncoder, SnmpInt, SnmpNull, SnmpOid, SnmpSequence};
use crate::buf::Buffer;
use crate::error::SnmpError;
use nom::IResult;

const DOUBLE_ZEROES: [u8; 6] = [2u8, 1, 0, 2, 1, 0];

pub(crate) struct SnmpGet {
    pub(crate) request_id: i64,
    pub(crate) vars: Vec<SnmpOid>,
}

impl<'a> TryFrom<&'a [u8]> for SnmpGet {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // Request id
        let (tail, request_id) = SnmpInt::from_ber(value)?;
        // error status, must be 0
        let (tail, error_status) = SnmpInt::from_ber(tail)?;
        if !error_status.is_zero() {
            return Err(SnmpError::InvalidPdu);
        }
        // error index, must be 0
        let (tail, error_index) = SnmpInt::from_ber(tail)?;
        if !error_index.is_zero() {
            return Err(SnmpError::InvalidPdu);
        }
        // varbinds
        let (tail, vb) = SnmpSequence::from_ber(tail)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        let mut v_tail = vb.0;
        let mut vars = Vec::<SnmpOid>::new();
        while !v_tail.is_empty() {
            let (rest, oid) = SnmpGet::parse_var(v_tail)?;
            vars.push(oid);
            v_tail = rest;
        }
        Ok(SnmpGet {
            request_id: request_id.into(),
            vars,
        })
    }
}

impl BerEncoder for SnmpGet {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
        // Push all vars in the reversed order
        let null = SnmpNull {};
        for oid in self.vars.iter().rev() {
            let start = buf.len();
            // Trailing null
            null.push_ber(buf)?;
            // OID
            oid.push_ber(buf)?;
            // Enclosing sequence
            buf.push_ber_len(buf.len() - start)?;
            // Sequence tag
            buf.push_u8(0x30)?;
        }
        // Enclosing sequence for varbinds
        // Spans for the end
        buf.push_ber_len(buf.len())?;
        buf.push_u8(0x30)?;
        // Error index + error status, both zeroes
        buf.push(&DOUBLE_ZEROES)?;
        // Request id
        let r_id: SnmpInt = self.request_id.into();
        r_id.push_ber(buf)?;
        Ok(())
    }
}

impl SnmpGet {
    fn parse_var(i: &[u8]) -> IResult<&[u8], SnmpOid, SnmpError> {
        // Parse enclosing sequence
        let (rest, vs) = SnmpSequence::from_ber(i)?;
        // Parse oid
        let (tail, oid) = SnmpOid::from_ber(vs.0)?;
        // Parse null
        let (_, _) = SnmpNull::from_ber(tail)?;
        Ok((rest, oid))
    }
}
