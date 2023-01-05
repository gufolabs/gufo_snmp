// ------------------------------------------------------------------------
// Gufo Snmp: GETRESPONSE PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::var::SnmpVar;
use crate::ber::{BerDecoder, SnmpInt, SnmpSequence};
use crate::error::SnmpError;

pub(crate) struct SnmpGetResponse<'a> {
    request_id: i64,
    error_status: i64,
    error_index: i64,
    vars: Vec<SnmpVar<'a>>,
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
            let (rest, oid) = SnmpVar::from_ber(v_tail)?;
            vars.push(oid);
            v_tail = rest;
        }
        Ok(SnmpGetResponse {
            request_id: request_id.as_i64(),
            error_status: error_status.as_i64(),
            error_index: error_index.as_i64(),
            vars,
        })
    }
}
