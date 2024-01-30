// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 No Auth
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::SnmpAuth;
use crate::error::SnmpResult;

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
    fn sign(&self, _data: &mut [u8], _offset: usize) -> SnmpResult<()> {
        Ok(())
    }
}
