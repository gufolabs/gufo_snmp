// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 Auth primitives
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod md5;
mod sha1;

pub use crate::error::{SnmpError, SnmpResult};
pub use md5::Md5;
pub use sha1::Sha1;

pub const NO_AUTH: u8 = 0;
pub const MD5_AUTH: u8 = 1;
pub const SHA1_AUTH: u8 = 2;

pub enum AuthKey {
    NoAuth,
    Md5(Md5),
    Sha1(Sha1),
}

pub trait SnmpAuth {
    // Convert key to localized key
    fn localize(&mut self, engine_id: &[u8]);
    // Calculate and place signature
    fn sign(&self, whole_msg: &mut [u8], offset: usize);
}

const PLACEHOLDER0: [u8; 0] = [];
const PLACEHOLDER12: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const PLACEHOLDER_MASK12: [u8; 14] = [4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

impl AuthKey {
    pub fn new(code: u8, key: Vec<u8>) -> Result<AuthKey, SnmpError> {
        match code {
            NO_AUTH => Ok(AuthKey::NoAuth),
            MD5_AUTH => Ok(AuthKey::Md5(Md5::new(key))),
            SHA1_AUTH => Ok(AuthKey::Sha1(Sha1::new(key))),
            _ => Err(SnmpError::InvalidVersion(code)),
        }
    }
    pub fn has_auth(&self) -> bool {
        !matches!(self, AuthKey::NoAuth)
    }
    pub fn placeholder(&self) -> &'static [u8] {
        match self {
            AuthKey::NoAuth => &PLACEHOLDER0,
            AuthKey::Md5(_) => &PLACEHOLDER12,
            AuthKey::Sha1(_) => &PLACEHOLDER12,
        }
    }
    pub fn localize(&mut self, engine_id: &[u8]) {
        match self {
            AuthKey::NoAuth => {}
            AuthKey::Md5(x) => x.localize(engine_id),
            AuthKey::Sha1(x) => x.localize(engine_id),
        }
    }
    pub fn sign(&self, whole_msg: &mut [u8]) -> SnmpResult<()> {
        match self {
            AuthKey::NoAuth => Ok(()),
            AuthKey::Md5(x) => match AuthKey::find_placeholder12_offset(whole_msg) {
                Some(idx) => {
                    x.sign(whole_msg, idx + 2);
                    Ok(())
                }
                None => Err(SnmpError::InvalidData),
            },
            AuthKey::Sha1(x) => match AuthKey::find_placeholder12_offset(whole_msg) {
                Some(idx) => {
                    x.sign(whole_msg, idx + 2);
                    Ok(())
                }
                None => Err(SnmpError::InvalidData),
            },
        }
    }
    fn find_placeholder12_offset(input: &[u8]) -> Option<usize> {
        for (i, window) in input.windows(PLACEHOLDER_MASK12.len()).enumerate() {
            if window == PLACEHOLDER_MASK12 {
                return Some(i);
            }
        }
        None
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
        let master_key = vec![117u8, 115, 101, 114, 49, 48, 107, 101, 121]; // user10key
        let engine_id = [128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100];
        let mut key = Md5::new(master_key);
        key.localize(&engine_id);
        let auth_key = AuthKey::Md5(key);
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
        let master_key = vec![117u8, 115, 101, 114, 50, 48, 107, 101, 121]; // user20key
        let engine_id = [128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100];
        let mut key = Sha1::new(master_key);
        key.localize(&engine_id);
        let auth_key = AuthKey::Sha1(key);
        auth_key.sign(&mut whole_msg)?;
        assert_eq!(whole_msg, expected);
        Ok(())
    }
}
