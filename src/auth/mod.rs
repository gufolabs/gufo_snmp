// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 Auth primitives
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod digest;
mod noauth;
use enum_dispatch::enum_dispatch;
use md5::Md5;
use sha1::Sha1;

pub use crate::error::{SnmpError, SnmpResult};
pub use digest::DigestAuth;
pub use noauth::NoAuth;

pub const NO_AUTH: u8 = 0;
pub const MD5_AUTH: u8 = 1;
pub const SHA1_AUTH: u8 = 2;

pub type Md5AuthKey = DigestAuth<Md5, 16, 12>;
pub type Sha1AuthKey = DigestAuth<Sha1, 20, 12>;

#[enum_dispatch(SnmpAuth)]
pub enum AuthKey {
    NoAuth(NoAuth),
    Md5(Md5AuthKey),
    Sha1(Sha1AuthKey),
}

#[enum_dispatch]
pub trait SnmpAuth {
    // Localized key
    fn from_localized(&mut self, key: &[u8]);
    // Master key, localized internally
    fn from_master(&mut self, key: &[u8], locality: &[u8]);
    // Convert key to localized key and write to output
    fn localize(&self, key: &[u8], locality: &[u8], out: &mut [u8]);
    // Check if method provides auth
    fn has_auth(&self) -> bool;
    // Returns zero-filled placeholder
    fn placeholder(&self) -> &'static [u8];
    // Find signature place
    fn find_placeholder_offset(&self, whole_msg: &[u8]) -> Option<usize>;
    // Calculate and place signature
    fn sign_and_update(&self, whole_msg: &mut [u8], offset: usize);
    //
    fn sign(&self, whole_msg: &mut [u8]) -> SnmpResult<()> {
        match self.find_placeholder_offset(whole_msg) {
            Some(offset) => {
                self.sign_and_update(whole_msg, offset);
                Ok(())
            }
            None => Err(SnmpError::InvalidData),
        }
    }
}

impl AuthKey {
    pub fn new(code: u8) -> SnmpResult<AuthKey> {
        Ok(match code {
            NO_AUTH => AuthKey::NoAuth(NoAuth),
            MD5_AUTH => AuthKey::Md5(Md5AuthKey::default()),
            SHA1_AUTH => AuthKey::Sha1(Sha1AuthKey::default()),
            _ => return Err(SnmpError::InvalidVersion(code)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_sign() -> SnmpResult<()> {
        let mut whole_msg = [
            48u8, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
            47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
            1, 0, 4, 6, 117, 115, 101, 114, 49, 48, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 4, 0, 160, 28,
            2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8, 43, 6, 1, 2, 1, 1, 4, 0,
            5, 0,
        ];
        let expected = [
            48u8, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
            47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
            1, 0, 4, 6, 117, 115, 101, 114, 49, 48, 4, 12, 18, 138, 173, 156, 223, 188, 26, 178,
            137, 113, 25, 22, 4, 0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54,
            116, 100, 4, 0, 160, 28, 2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8,
            43, 6, 1, 2, 1, 1, 4, 0, 5, 0,
        ];
        let master_key = [117u8, 115, 101, 114, 49, 48, 107, 101, 121]; // user10key
        let engine_id = [128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100];
        let mut auth_key = Md5AuthKey::default();
        auth_key.from_master(&master_key, &engine_id);
        auth_key.sign(&mut whole_msg)?;
        assert_eq!(whole_msg, expected);
        Ok(())
    }
    #[test]
    fn test_sha1_sign() -> SnmpResult<()> {
        let mut whole_msg = [
            48, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
            47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
            1, 0, 4, 6, 117, 115, 101, 114, 50, 48, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 4, 0, 160, 28,
            2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8, 43, 6, 1, 2, 1, 1, 4, 0,
            5, 0,
        ];
        let expected = [
            48, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
            47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
            1, 0, 4, 6, 117, 115, 101, 114, 50, 48, 4, 12, 8, 126, 173, 253, 67, 91, 150, 217, 19,
            212, 52, 193, 4, 0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116,
            100, 4, 0, 160, 28, 2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8, 43,
            6, 1, 2, 1, 1, 4, 0, 5, 0,
        ];
        let master_key = [117u8, 115, 101, 114, 50, 48, 107, 101, 121]; // user20key
        let engine_id = [128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100];
        let mut auth_key = Sha1AuthKey::default();
        auth_key.from_master(&master_key, &engine_id);
        auth_key.sign(&mut whole_msg)?;
        assert_eq!(whole_msg, expected);
        Ok(())
    }
}
