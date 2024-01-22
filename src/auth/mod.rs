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
    fn as_localized(&mut self, key: &[u8]);
    // Master key, localized internally
    fn as_master(&mut self, key: &[u8], locality: &[u8]);
    // Password, converted to master and localized internally
    fn as_password(&mut self, password: &[u8], locality: &[u8]);
    // Convert master key to localized key and write to output
    fn localize(&self, key: &[u8], locality: &[u8], out: &mut [u8]);
    // Convert password to master key
    fn password_to_master(&self, password: &[u8], out: &mut [u8]);
    // Get slice with key
    fn get_key(&self) -> &[u8];
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

// - - X X    X X X X
const KT_ALG_MASK: u8 = 0x3f;
// X X - -    - - - -
const KT_TYPE_MASK: u8 = 0xc0;
// 0 0 - -    - - - -
const KT_PASSWORD: u8 = 0;
// 0 1 - -    - - - -
const KT_MASTER: u8 = 0x40;
// 1 0 - -    - - - -
const KT_LOCALIZED: u8 = 0x80;

impl AuthKey {
    pub fn new(code: u8) -> SnmpResult<AuthKey> {
        Ok(match code & KT_ALG_MASK {
            NO_AUTH => AuthKey::NoAuth(NoAuth),
            MD5_AUTH => AuthKey::Md5(Md5AuthKey::default()),
            SHA1_AUTH => AuthKey::Sha1(Sha1AuthKey::default()),
            _ => return Err(SnmpError::InvalidVersion(code)),
        })
    }
    pub fn as_key_type(&mut self, alg: u8, key: &[u8], engine_id: &[u8]) -> SnmpResult<()> {
        if self.has_auth() {
            match alg & KT_TYPE_MASK {
                KT_PASSWORD => self.as_password(key, engine_id),
                KT_MASTER => self.as_master(key, engine_id),
                KT_LOCALIZED => self.as_localized(key),
                _ => return Err(SnmpError::InvalidKey),
            }
        }
        Ok(())
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
    #[test]
    fn test_md5_from_password() -> SnmpResult<()> {
        let auth_key = Md5AuthKey::default();
        let password = b"maplesyrup";
        let engine_id = [00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 02];
        let expected1 = [
            0x9f, 0xaf, 0x32, 0x83, 0x88, 0x4e, 0x92, 0x83, 0x4e, 0xbc, 0x98, 0x47, 0xd8, 0xed,
            0xd9, 0x63,
        ];
        let expected2 = [
            0x52, 0x6f, 0x5e, 0xed, 0x9f, 0xcc, 0xe2, 0x6f, 0x89, 0x64, 0xc2, 0x93, 0x07, 0x87,
            0xd8, 0x2b,
        ];
        let mut out1 = [0u8; 16];
        auth_key.password_to_master(password, &mut out1);
        assert_eq!(out1, expected1);
        // localize
        let mut out2 = [0u8; 16];
        auth_key.localize(&out1, &engine_id, &mut out2);
        assert_eq!(out2, expected2);
        Ok(())
    }
    #[test]
    fn test_sha_from_password() -> SnmpResult<()> {
        let auth_key = Sha1AuthKey::default();
        let password = b"maplesyrup";
        let engine_id = [00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 02];
        let expected1 = [
            0x9f, 0xb5, 0xcc, 0x03, 0x81, 0x49, 0x7b, 0x37, 0x93, 0x52, 0x89, 0x39, 0xff, 0x78,
            0x8d, 0x5d, 0x79, 0x14, 0x52, 0x11,
        ];
        let expected2 = [
            0x66, 0x95, 0xfe, 0xbc, 0x92, 0x88, 0xe3, 0x62, 0x82, 0x23, 0x5f, 0xc7, 0x15, 0x1f,
            0x12, 0x84, 0x97, 0xb3, 0x8f, 0x3f,
        ];
        let mut out1 = [0u8; 20];
        auth_key.password_to_master(password, &mut out1);
        assert_eq!(out1, expected1);
        let mut out2 = [0u8; 20];
        auth_key.localize(&out1, &engine_id, &mut out2);
        assert_eq!(out2, expected2);
        Ok(())
    }
}
