// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 No Auth
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::SnmpAuth;

#[derive(Default)]
pub struct NoAuth;

const PLACEHOLDER: [u8; 0] = [];

impl SnmpAuth for NoAuth {
    fn from_localized(&mut self, _key: &[u8]) {}
    fn from_master(&mut self, _key: &[u8], _locality: &[u8]) {}
    fn has_auth(&self) -> bool {
        false
    }
    fn placeholder(&self) -> &'static [u8] {
        &PLACEHOLDER
    }
    fn localize(&self, _key: &[u8], _locality: &[u8], _out: &mut [u8]) {}
    fn find_placeholder_offset(&self, _whole_msg: &[u8]) -> Option<usize> {
        None
    }
    fn sign_and_update(&self, _whole_msg: &mut [u8], _offset: usize) {}
    fn sign(&self, _whole_msg: &mut [u8]) -> super::SnmpResult<()> {
        Ok(())
    }
}
