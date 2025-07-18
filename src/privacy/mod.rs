// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 privacy primitives
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod aes128;
mod des;
mod nopriv;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};
use aes128::Aes128Key;
use des::DesKey;
use enum_dispatch::enum_dispatch;
use nopriv::NoPriv;

#[enum_dispatch(SnmpPriv)]
pub enum PrivKey {
    NoPriv(NoPriv),
    Des(DesKey),
    Aes128(Aes128Key),
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

const NO_PRIV: u8 = 0;
const DES: u8 = 1;
const AES128: u8 = 2;
// - - X X    X X X X
const KT_ALG_MASK: u8 = 0x3f;

impl PrivKey {
    pub fn new(code: u8) -> SnmpResult<PrivKey> {
        Ok(match code & KT_ALG_MASK {
            NO_PRIV => PrivKey::NoPriv(NoPriv),
            DES => PrivKey::Des(DesKey::default()),
            AES128 => PrivKey::Aes128(Aes128Key::default()),
            _ => return Err(SnmpError::InvalidVersion(code)),
        })
    }
}
