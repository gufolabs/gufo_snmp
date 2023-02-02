// ------------------------------------------------------------------------
// Gufo SNMP: GETRESPONSE PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::value::SnmpValue;
use crate::ber::{BerDecoder, SnmpInt, SnmpOid, SnmpSequence};
use crate::error::SnmpError;

#[allow(dead_code)]
pub(crate) struct SnmpGetResponse<'a> {
    pub(crate) request_id: i64,
    pub(crate) error_status: i64,
    pub(crate) error_index: i64,
    pub(crate) vars: Vec<SnmpVar<'a>>,
}

pub(crate) struct SnmpVar<'a> {
    pub(crate) oid: SnmpOid,
    pub(crate) value: SnmpValue<'a>,
}

impl<'a> TryFrom<&'a [u8]> for SnmpGetResponse<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // Request id
        let (tail, request_id) = SnmpInt::from_ber(value)?;
        // error status
        let (tail, error_status) = SnmpInt::from_ber(tail)?;
        // error index
        let (tail, error_index) = SnmpInt::from_ber(tail)?;
        // varbinds
        let (tail, vb) = SnmpSequence::from_ber(tail)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        let mut v_tail = vb.0;
        let mut vars = Vec::new();
        while !v_tail.is_empty() {
            // Parse enclosing sequence
            let (rest, vs) = SnmpSequence::from_ber(v_tail)?;
            // Parse oid
            let (tail, oid) = SnmpOid::from_ber(vs.0)?;
            // Parse value
            let (_, value) = SnmpValue::from_ber(tail)?;
            //<
            vars.push(SnmpVar { oid, value });
            // Shift to the next
            v_tail = rest;
        }
        Ok(SnmpGetResponse {
            request_id: request_id.into(),
            error_status: error_status.into(),
            error_index: error_index.into(),
            vars,
        })
    }
}
