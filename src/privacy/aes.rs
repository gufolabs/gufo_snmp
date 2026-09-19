// ------------------------------------------------------------------------
// Gufo SNMP: AES privacy mode
// ------------------------------------------------------------------------
// Copyright (C) 2023-26, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{SnmpPriv, get_padded_len};
use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};
use aes::{Aes128, Aes192, Aes256};
use cfb_mode::{Decryptor, Encryptor};
use cipher::{
    AsyncStreamCipher, BlockCipher, BlockEncryptMut, KeyInit, KeyIvInit, block_padding::NoPadding,
};
use rand::Rng;
use std::marker::PhantomData;

const BLOCK_SIZE: usize = 16;

pub struct AesKey<C, const KEY_LENGTH: usize> {
    key: [u8; KEY_LENGTH],
    priv_params: [u8; BLOCK_SIZE],
    salt_value: u64,
    buf: Buffer,
    _cipher: PhantomData<C>,
}

// Define key types
pub type Aes128Key = AesKey<Aes128, 16>;
pub type Aes192Key = AesKey<Aes192, 24>;
pub type Aes256Key = AesKey<Aes256, 32>;

impl<C, const KEY_LENGTH: usize> Default for AesKey<C, KEY_LENGTH> {
    fn default() -> Self {
        Self {
            key: [0; KEY_LENGTH],
            priv_params: [0; BLOCK_SIZE],
            salt_value: 0,
            buf: Buffer::default(),
            _cipher: PhantomData,
        }
    }
}

impl<C, const KEY_LENGTH: usize> AesKey<C, KEY_LENGTH> {
    fn set_key(&mut self, key: &[u8]) -> SnmpResult<()> {
        if key.len() < KEY_LENGTH {
            return Err(SnmpError::InvalidKey);
        }
        self.key.copy_from_slice(&key[..KEY_LENGTH]);
        let mut rng = rand::rng();
        self.salt_value = rng.random();
        Ok(())
    }

    fn make_iv(&mut self, boots: u32, time: u32) {
        self.priv_params[..4].copy_from_slice(&boots.to_be_bytes());
        self.priv_params[4..8].copy_from_slice(&time.to_be_bytes());
        self.priv_params[8..].copy_from_slice(&self.salt_value.to_be_bytes());
        self.salt_value = self.salt_value.wrapping_add(1);
    }

    fn prepare_data(&mut self, pdu: &ScopedPdu) -> SnmpResult<usize> {
        self.buf.reset();
        self.buf.skip(BLOCK_SIZE);
        pdu.push_ber(&mut self.buf)?;
        let scoped_len = self.buf.len() - BLOCK_SIZE;
        let padded_len = get_padded_len(scoped_len, BLOCK_SIZE);
        let pad_len = padded_len - scoped_len;
        if pad_len > 0 {
            self.buf.fill_u8(scoped_len, pad_len as u8, pad_len)?;
        }
        Ok(padded_len)
    }
}

impl<C, const KEY_LENGTH: usize> SnmpPriv for AesKey<C, KEY_LENGTH>
where
    C: BlockEncryptMut + BlockCipher + KeyInit,
{
    fn as_localized(&mut self, key: &[u8]) -> SnmpResult<()> {
        self.set_key(key)
    }

    fn has_priv(&self) -> bool {
        true
    }

    fn encrypt<'a>(
        &'a mut self,
        pdu: &ScopedPdu,
        boots: u32,
        time: u32,
    ) -> SnmpResult<(&'a [u8], &'a [u8])> {
        self.make_iv(boots, time);
        let padded_len = self.prepare_data(pdu)?;
        let encryptor = Encryptor::<C>::new_from_slices(&self.key, &self.priv_params)
            .map_err(|_| SnmpError::InvalidKey)?;
        let b = self.buf.data_mut();
        encryptor
            .encrypt_padded_mut::<NoPadding>(&mut b[..padded_len], padded_len)
            .map_err(|_| SnmpError::InvalidKey)?;
        Ok((&b[..padded_len], &self.priv_params[8..]))
    }

    fn decrypt<'a: 'c, 'b, 'c>(
        &'a mut self,
        data: &'b [u8],
        usm: &'b UsmParameters<'b>,
    ) -> SnmpResult<ScopedPdu<'c>> {
        let mut iv = [0u8; BLOCK_SIZE];
        iv[..4].copy_from_slice(&(usm.engine_boots as u32).to_be_bytes());
        iv[4..8].copy_from_slice(&(usm.engine_time as u32).to_be_bytes());
        iv[8..].copy_from_slice(usm.privacy_params);
        let decryptor =
            Decryptor::<C>::new_from_slices(&self.key, &iv).map_err(|_| SnmpError::InvalidKey)?;
        self.buf.reset();
        self.buf.skip(data.len());
        let b = self.buf.data_mut();
        decryptor
            .decrypt_b2b(data, b)
            .map_err(|_| SnmpError::InvalidKey)?;
        let scoped_pdu = ScopedPdu::try_from(self.buf.data())?;
        Ok(scoped_pdu)
    }
}
