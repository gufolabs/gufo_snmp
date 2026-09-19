// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 privacy primitives
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod aes;
mod des;
mod nopriv;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};
use aes::{Aes128Key, Aes192Key, Aes256Key};
use des::DesKey;
use enum_dispatch::enum_dispatch;
use nopriv::NoPriv;

#[enum_dispatch(SnmpPriv)]
pub enum PrivKey {
    NoPriv(NoPriv),
    Des(DesKey),
    Aes128(Aes128Key),
    Aes192(Aes192Key),
    Aes256(Aes256Key),
}

#[enum_dispatch]
pub trait SnmpPriv {
    // Localized key
    fn as_localized(&mut self, key: &[u8]) -> SnmpResult<()>;
    //
    fn has_priv(&self) -> bool;
    // Encrypt data.
    // Returns (encrypted data, priv parameters)
    fn encrypt<'a>(
        &'a mut self,
        pdu: &ScopedPdu,
        boots: u32,
        time: u32,
    ) -> SnmpResult<(&'a [u8], &'a [u8])>;
    // Decrypt data
    fn decrypt<'a: 'c, 'b, 'c>(
        &'a mut self,
        data: &'b [u8],
        usm: &'b UsmParameters<'b>,
    ) -> SnmpResult<ScopedPdu<'c>>;
}

#[inline]
fn get_padded_len(buf_len: usize, block_size: usize) -> usize {
    let rem = buf_len % block_size;
    if rem == 0 {
        buf_len
    } else {
        buf_len - rem + block_size
    }
}

// - - X X    X X X X
const KT_ALG_MASK: u8 = 0x3f;

impl TryFrom<u8> for PrivKey {
    type Error = SnmpError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value & KT_ALG_MASK {
            0 => PrivKey::NoPriv(NoPriv),
            1 => PrivKey::Des(DesKey::default()),
            2 => PrivKey::Aes128(Aes128Key::default()),
            3 => PrivKey::Aes192(Aes192Key::default()),
            4 => PrivKey::Aes256(Aes256Key::default()),
            _ => return Err(SnmpError::InvalidVersion(value)),
        })
    }
}
