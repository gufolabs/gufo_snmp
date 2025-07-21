// ------------------------------------------------------------------------
// Gufo SNMP: AES128 mode
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{SnmpPriv, get_padded_len};
use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};
use aes::Aes128;
use cfb_mode::{Decryptor, Encryptor};
use cipher::{AsyncStreamCipher, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use rand::Rng;

const KEY_LENGTH: usize = 16;
const BLOCK_SIZE: usize = 16;

type Aes128CfbEncryptor = Encryptor<Aes128>;
type Aes128CfbDecrypror = Decryptor<Aes128>;

#[derive(Default)]
pub struct Aes128Key {
    key: [u8; KEY_LENGTH],
    priv_params: [u8; KEY_LENGTH],
    salt_value: u64,
    buf: Buffer,
}

impl SnmpPriv for Aes128Key {
    fn as_localized(&mut self, key: &[u8]) -> SnmpResult<()> {
        if key.len() < KEY_LENGTH {
            return Err(SnmpError::InvalidKey);
        }
        self.key.copy_from_slice(&key[..KEY_LENGTH]);
        let mut rng = rand::rng();
        self.salt_value = rng.random();
        Ok(())
    }
    fn has_priv(&self) -> bool {
        true
    }
    // Returns data, priv parameters
    fn encrypt<'a>(
        &'a mut self,
        pdu: &ScopedPdu,
        boots: u32,
        time: u32,
    ) -> SnmpResult<(&'a [u8], &'a [u8])> {
        // Fill IV
        self.priv_params[..4].clone_from_slice(&boots.to_be_bytes());
        self.priv_params[4..8].clone_from_slice(&time.to_be_bytes());
        self.priv_params[8..].clone_from_slice(&self.salt_value.to_be_bytes());
        self.salt_value = self.salt_value.wrapping_add(1);
        // Add padding
        self.buf.reset();
        self.buf.skip(BLOCK_SIZE);
        // Serialize
        pdu.push_ber(&mut self.buf)?;
        // Calculate length
        let scoped_len = self.buf.len() - BLOCK_SIZE;
        let padded_len = get_padded_len(scoped_len, BLOCK_SIZE);
        // Fill padding
        let pad_len = padded_len - scoped_len;
        if pad_len > 0 {
            self.buf.fill_u8(scoped_len, pad_len as u8, pad_len)?;
        }
        // Encrypt
        let encryptor = Aes128CfbEncryptor::new_from_slices(&self.key, &self.priv_params)
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
        // Get IV
        let mut iv = [0u8; 16];
        iv[..4].clone_from_slice(&(usm.engine_boots as u32).to_be_bytes());
        iv[4..8].clone_from_slice(&(usm.engine_time as u32).to_be_bytes());
        iv[8..].clone_from_slice(usm.privacy_params);
        // Decrypt
        let decryptor = Aes128CfbDecrypror::new_from_slices(&self.key, &iv)
            .map_err(|_| SnmpError::InvalidKey)?;
        self.buf.reset();
        self.buf.skip(data.len());
        let b = self.buf.data_mut();
        decryptor
            .decrypt_b2b(data, b)
            .map_err(|_| SnmpError::InvalidKey)?;
        // Decode
        let scoped_pdu = ScopedPdu::try_from(self.buf.data())?;
        Ok(scoped_pdu)
    }
}
