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
    fn as_localized(&mut self, _key: &[u8]) {}
    fn as_master(&mut self, _key: &[u8], _locality: &[u8]) {}
    fn as_password(&mut self, _password: &[u8], _locality: &[u8]) {}
    fn get_key_size(&self) -> usize {
        0
    }
    fn get_key(&self) -> &[u8] {
        &PLACEHOLDER
    }
    fn has_auth(&self) -> bool {
        false
    }
    fn placeholder(&self) -> &'static [u8] {
        &PLACEHOLDER
    }
    fn localize(&self, _key: &[u8], _locality: &[u8], _out: &mut [u8]) {}
    fn password_to_master(&self, _password: &[u8], _out: &mut [u8]) {}
    fn find_placeholder_offset(&self, _whole_msg: &[u8]) -> Option<usize> {
        None
    }
    fn sign_and_update(&self, _whole_msg: &mut [u8], _offset: usize) {}
    fn sign(&self, _whole_msg: &mut [u8]) -> super::SnmpResult<()> {
        Ok(())
    }
}
