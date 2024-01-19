// ------------------------------------------------------------------------
// Gufo SNMP: No privacy implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::SnmpPriv;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};

#[derive(Default)]
pub struct NoPriv;

impl SnmpPriv for NoPriv {
    fn from_localized(&mut self, _key: &[u8]) -> SnmpResult<()> {
        Ok(())
    }
    fn has_priv(&self) -> bool {
        false
    }
    fn encrypt<'a>(
        &'a mut self,
        _pdu: &ScopedPdu,
        _boots: u32,
        _time: u32,
    ) -> SnmpResult<(&'a [u8], &'a [u8])> {
        Err(SnmpError::NotImplemented)
    }
    fn decrypt<'a: 'c, 'b, 'c>(
        &'a mut self,
        _data: &'b [u8],
        _usm: &'b UsmParameters<'b>,
    ) -> SnmpResult<ScopedPdu<'c>> {
        Err(SnmpError::NotImplemented)
    }
}
