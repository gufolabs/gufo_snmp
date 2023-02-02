// ------------------------------------------------------------------------
// Gufo SNMP: GETRESPONSE PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::value::SnmpValue;
use crate::ber::{
    BerDecoder, SnmpInt, SnmpOid, SnmpRelativeOid, SnmpSequence, TAG_OBJECT_ID, TAG_RELATIVE_OID,
};
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
        let mut vars: Vec<SnmpVar> = Vec::new();
        while !v_tail.is_empty() {
            // Parse enclosing sequence
            let (rest, vs) = SnmpSequence::from_ber(v_tail)?;
            // Parse oid. May be either absolute or relative
            let (tail, oid) = match vs.0[0] as usize {
                TAG_OBJECT_ID => SnmpOid::from_ber(vs.0)?,
                TAG_RELATIVE_OID => {
                    if vars.is_empty() {
                        // Relative oid must follow absolute one
                        return Err(SnmpError::UnexpectedTag);
                    }
                    // Parse relative oid
                    let (t, r_oid) = SnmpRelativeOid::from_ber(vs.0)?;
                    // Apply relative oid
                    (t, r_oid.normalize(&vars[vars.len() - 1].oid))
                }
                _ => return Err(SnmpError::UnexpectedTag),
            };
            // Parse value
            let (_, value) = SnmpValue::from_ber(tail)?;
            // Append an item
            vars.push(SnmpVar { oid, value });
            // Shift to the next var
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
