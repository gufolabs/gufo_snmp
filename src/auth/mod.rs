// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 Auth primitives
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod blumenthal;
mod cisco;
mod digest;
mod noauth;
use enum_dispatch::enum_dispatch;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512};

pub use crate::error::{SnmpError, SnmpResult};
pub use blumenthal::DigestAuthWithBlumenthal;
pub use cisco::DigestAuthWithCisco;
pub use digest::DigestAuth;
pub use noauth::NoAuth;

pub type Md5AuthKey = DigestAuth<Md5, 16, 12, 64>;
pub type Md5BlumenthalAuthKey = DigestAuthWithBlumenthal<Md5, 16, 12, 32, 64>;
pub type Md5CiscoAuthKey = DigestAuthWithCisco<Md5, 16, 12, 32, 64>;
pub type Sha1AuthKey = DigestAuth<Sha1, 20, 12, 64>;
pub type Sha1BlumenthalAuthKey = DigestAuthWithBlumenthal<Sha1, 20, 12, 32, 64>;
pub type Sha1CiscoAuthKey = DigestAuthWithCisco<Sha1, 20, 12, 32, 64>;
pub type Sha224AuthKey = DigestAuth<Sha224, 28, 16, 64>;
pub type Sha224BlumenthalAuthKey = DigestAuthWithBlumenthal<Sha224, 28, 16, 32, 64>;
pub type Sha224CiscoAuthKey = DigestAuthWithCisco<Sha224, 28, 16, 32, 64>;
pub type Sha256AuthKey = DigestAuth<Sha256, 32, 24, 64>;
pub type Sha384AuthKey = DigestAuth<Sha384, 48, 32, 128>;
pub type Sha512AuthKey = DigestAuth<Sha512, 64, 48, 128>;

const ZEROES: [u8; 128] = [0; 128];

#[enum_dispatch(SnmpAuth)]
pub enum AuthKey {
    NoAuth(NoAuth),
    Md5(Md5AuthKey),
    Md5Blumenthal(Md5BlumenthalAuthKey),
    Md5Cisco(Md5CiscoAuthKey),
    Sha1(Sha1AuthKey),
    Sha1Blumenthal(Sha1BlumenthalAuthKey),
    Sha1Cisco(Sha1CiscoAuthKey),
    Sha224(Sha224AuthKey),
    Sha224Blumenthal(Sha224BlumenthalAuthKey),
    Sha224Cisco(Sha224CiscoAuthKey),
    Sha256(Sha256AuthKey),
    Sha384(Sha384AuthKey),
    Sha512(Sha512AuthKey),
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
    // Get key size
    fn get_key_size(&self) -> usize;
    // Get slice with key
    fn get_key(&self) -> &[u8];
    // Check if method provides auth
    fn has_auth(&self) -> bool;
    // Returns zero-filled placeholder
    fn placeholder(&self) -> &'static [u8];
    // Sign data in buffer
    fn sign(&self, data: &mut [u8], offset: usize) -> SnmpResult<()>;
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

impl TryFrom<u8> for AuthKey {
    type Error = SnmpError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value & KT_ALG_MASK {
            0 => AuthKey::NoAuth(NoAuth),
            1 => AuthKey::Md5(Md5AuthKey::default()),
            2 => AuthKey::Md5Blumenthal(Md5BlumenthalAuthKey::default()),
            3 => AuthKey::Md5Cisco(Md5CiscoAuthKey::default()),
            4 => AuthKey::Sha1(Sha1AuthKey::default()),
            5 => AuthKey::Sha1Blumenthal(Sha1BlumenthalAuthKey::default()),
            6 => AuthKey::Sha1Cisco(Sha1CiscoAuthKey::default()),
            7 => AuthKey::Sha224(Sha224AuthKey::default()),
            8 => AuthKey::Sha224Blumenthal(Sha224BlumenthalAuthKey::default()),
            9 => AuthKey::Sha224Cisco(Sha224CiscoAuthKey::default()),
            10 => AuthKey::Sha256(Sha256AuthKey::default()),
            11 => AuthKey::Sha384(Sha384AuthKey::default()),
            12 => AuthKey::Sha512(Sha512AuthKey::default()),
            _ => return Err(SnmpError::InvalidVersion(value)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_key_try_from() {
        assert!(matches!(AuthKey::try_from(0), Ok(AuthKey::NoAuth(_))));
        assert!(matches!(AuthKey::try_from(1), Ok(AuthKey::Md5(_))));
        assert!(matches!(
            AuthKey::try_from(2),
            Ok(AuthKey::Md5Blumenthal(_))
        ));
        assert!(matches!(AuthKey::try_from(3), Ok(AuthKey::Md5Cisco(_))));
        assert!(matches!(AuthKey::try_from(4), Ok(AuthKey::Sha1(_))));
        assert!(matches!(
            AuthKey::try_from(5),
            Ok(AuthKey::Sha1Blumenthal(_))
        ));
        assert!(matches!(AuthKey::try_from(6), Ok(AuthKey::Sha1Cisco(_))));
        assert!(matches!(AuthKey::try_from(7), Ok(AuthKey::Sha224(_))));
        assert!(matches!(
            AuthKey::try_from(8),
            Ok(AuthKey::Sha224Blumenthal(_))
        ));
        assert!(matches!(AuthKey::try_from(9), Ok(AuthKey::Sha224Cisco(_))));
        assert!(matches!(AuthKey::try_from(10), Ok(AuthKey::Sha256(_))));
        assert!(matches!(AuthKey::try_from(11), Ok(AuthKey::Sha384(_))));
        assert!(matches!(AuthKey::try_from(12), Ok(AuthKey::Sha512(_))));
    }

    #[test]
    fn test_auth_key_try_from_with_key_type() {
        for value in 0..=12 {
            for key_type in [KT_PASSWORD, KT_MASTER, KT_LOCALIZED] {
                assert!(AuthKey::try_from(value | key_type).is_ok());
            }
        }
    }

    #[test]
    fn test_auth_key_try_from_invalid() {
        for value in 13..=KT_ALG_MASK {
            assert!(AuthKey::try_from(value).is_err());
        }
    }

    #[test]
    fn test_md5_sign() -> SnmpResult<()> {
        let mut whole_msg = [
            48, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
            47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
            1, 0, 4, 6, 117, 115, 101, 114, 49, 48, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 4, 0, 160, 28,
            2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8, 43, 6, 1, 2, 1, 1, 4, 0,
            5, 0,
        ];
        let offset = 58;
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
        auth_key.as_master(&master_key, &engine_id);
        auth_key.sign(&mut whole_msg, offset)?;
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
        let offset = 58;
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
        auth_key.as_master(&master_key, &engine_id);
        auth_key.sign(&mut whole_msg, offset)?;
        assert_eq!(whole_msg, expected);
        Ok(())
    }
    #[test]
    fn test_md5_from_password() -> SnmpResult<()> {
        let auth_key = Md5AuthKey::default();
        let password = b"maplesyrup";
        let engine_id = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
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
        let engine_id = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
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
    #[test]
    fn test_md5_blumenthal_key_expansion() -> SnmpResult<()> {
        let auth_key = Md5BlumenthalAuthKey::default();
        let localized_key = [
            0x52, 0x6f, 0x5e, 0xed, 0x9f, 0xcc, 0xe2, 0x6f, 0x89, 0x64, 0xc2, 0x93, 0x07, 0x87,
            0xd8, 0x2b,
        ];
        let expected = [
            0x52, 0x6f, 0x5e, 0xed, 0x9f, 0xcc, 0xe2, 0x6f, 0x89, 0x64, 0xc2, 0x93, 0x07, 0x87,
            0xd8, 0x2b, 0xfa, 0x24, 0xa9, 0x24, 0x67, 0x42, 0x6c, 0x2f, 0x4b, 0x09, 0x19, 0x2b,
            0xe1, 0x0d, 0xfa, 0xec,
        ];
        let mut auth_key = auth_key;
        auth_key.as_localized(&localized_key);
        assert_eq!(auth_key.get_key_size(), 32);
        assert_eq!(auth_key.get_key(), expected);
        Ok(())
    }

    #[test]
    fn test_sha1_blumenthal_key_expansion() -> SnmpResult<()> {
        let mut auth_key = Sha1BlumenthalAuthKey::default();
        let localized_key = [
            0x66, 0x95, 0xfe, 0xbc, 0x92, 0x88, 0xe3, 0x62, 0x82, 0x23, 0x5f, 0xc7, 0x15, 0x1f,
            0x12, 0x84, 0x97, 0xb3, 0x8f, 0x3f,
        ];
        let expected = [
            0x66, 0x95, 0xfe, 0xbc, 0x92, 0x88, 0xe3, 0x62, 0x82, 0x23, 0x5f, 0xc7, 0x15, 0x1f,
            0x12, 0x84, 0x97, 0xb3, 0x8f, 0x3f, 0x50, 0x5e, 0x07, 0xeb, 0x9a, 0xf2, 0x55, 0x68,
            0xfa, 0x1f, 0x5d, 0xbe,
        ];
        auth_key.as_localized(&localized_key);
        assert_eq!(auth_key.get_key_size(), 32);
        assert_eq!(auth_key.get_key(), expected);
        Ok(())
    }
    #[test]
    fn test_blumenthal_auth_key_dispatch() -> SnmpResult<()> {
        let mut md5 = AuthKey::try_from(2)?;
        let mut sha1 = AuthKey::try_from(5)?;
        let md5_key = [
            0x52, 0x6f, 0x5e, 0xed, 0x9f, 0xcc, 0xe2, 0x6f, 0x89, 0x64, 0xc2, 0x93, 0x07, 0x87,
            0xd8, 0x2b,
        ];
        let sha1_key = [
            0x66, 0x95, 0xfe, 0xbc, 0x92, 0x88, 0xe3, 0x62, 0x82, 0x23, 0x5f, 0xc7, 0x15, 0x1f,
            0x12, 0x84, 0x97, 0xb3, 0x8f, 0x3f,
        ];
        md5.as_localized(&md5_key);
        sha1.as_localized(&sha1_key);
        assert_eq!(md5.get_key_size(), 32);
        assert_eq!(sha1.get_key_size(), 32);
        Ok(())
    }
    #[test]
    fn test_md5_cisco_key_expansion() {
        let mut auth = Md5CiscoAuthKey::default();
        let engine_id = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
        auth.as_password(b"maplesyrup", &engine_id);
        assert_eq!(
            auth.get_key(),
            &[
                0x52, 0x6f, 0x5e, 0xed, 0x9f, 0xcc, 0xe2, 0x6f, 0x89, 0x64, 0xc2, 0x93, 0x07, 0x87,
                0xd8, 0x2b, 0x79, 0xef, 0xf4, 0x4a, 0x90, 0x65, 0x0e, 0xe0, 0xa3, 0xa4, 0x0a, 0xbf,
                0xac, 0x5a, 0xcc, 0x12,
            ]
        );
    }

    #[test]
    fn test_sha1_cisco_key_expansion() {
        let mut auth = Sha1CiscoAuthKey::default();
        let engine_id = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
        auth.as_password(b"maplesyrup", &engine_id);
        assert_eq!(
            auth.get_key(),
            &[
                0x66, 0x95, 0xfe, 0xbc, 0x92, 0x88, 0xe3, 0x62, 0x82, 0x23, 0x5f, 0xc7, 0x15, 0x1f,
                0x12, 0x84, 0x97, 0xb3, 0x8f, 0x3f, 0x9b, 0x8b, 0x6d, 0x78, 0x93, 0x6b, 0xa6, 0xe7,
                0xd1, 0x9d, 0xfd, 0x9c,
            ]
        );
    }

    #[test]
    fn test_cisco_auth_key_dispatch() {
        let auth = AuthKey::try_from(3).unwrap();
        assert!(matches!(auth, AuthKey::Md5Cisco(_)));

        let auth = AuthKey::try_from(6).unwrap();
        assert!(matches!(auth, AuthKey::Sha1Cisco(_)));
    }
}
