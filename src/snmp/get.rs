// ------------------------------------------------------------------------
// Gufo Snmp: GET PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{BerDecoder, SnmpInt, SnmpNull, SnmpOid, SnmpSequence};
use crate::error::SnmpError;
use nom::IResult;

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
        if error_status.as_i64() != 0 {
            return Err(SnmpError::InvalidPdu);
        }
        // error index, must be 0
        let (tail, error_index) = SnmpInt::from_ber(tail)?;
        if error_index.as_i64() != 0 {
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
            request_id: request_id.as_i64(),
            vars,
        })
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
