// ------------------------------------------------------------------------
// Gufo SNMP: DES mode
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{SnmpPriv, get_padded_len};
use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::msg::v3::{ScopedPdu, UsmParameters};
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use des::Des;
use rand::Rng;

const KEY_LENGTH: usize = 16;
const ENC_KEY_LENGTH: usize = 8;
const SALT_SIZE: usize = 8;
const BLOCK_SIZE: usize = 8;

type DesCbcEncryptor = Encryptor<Des>;
type DesCbcDecryptor = Decryptor<Des>;

#[derive(Default)]
pub struct DesKey {
    key: [u8; ENC_KEY_LENGTH],
    pre_iv: [u8; KEY_LENGTH - ENC_KEY_LENGTH],
    priv_params: [u8; SALT_SIZE],
    salt_value: u32,
    buf: Buffer,
}

impl SnmpPriv for DesKey {
    fn as_localized(&mut self, key: &[u8]) -> SnmpResult<()> {
        if key.len() < KEY_LENGTH {
            return Err(SnmpError::InvalidKey);
        }
        self.key.copy_from_slice(&key[..ENC_KEY_LENGTH]);
        self.pre_iv
            .copy_from_slice(&key[ENC_KEY_LENGTH..KEY_LENGTH]);
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
        _time: u32,
    ) -> SnmpResult<(&'a [u8], &'a [u8])> {
        // Calculate salt
        self.priv_params[..4].clone_from_slice(&boots.to_be_bytes());
        self.priv_params[4..].clone_from_slice(&self.salt_value.to_be_bytes());
        self.salt_value = self.salt_value.wrapping_add(1);
        // Get  IV
        let mut iv = [0u8; 8];
        for (idx, (x, y)) in self.priv_params.iter().zip(self.pre_iv.iter()).enumerate() {
            iv[idx] = x ^ y;
        }
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
        let encryptor =
            DesCbcEncryptor::new_from_slices(&self.key, &iv).map_err(|_| SnmpError::InvalidKey)?;
        let b = self.buf.data_mut();
        encryptor
            .encrypt_padded_mut::<NoPadding>(&mut b[..padded_len], padded_len)
            .map_err(|_| SnmpError::InvalidKey)?;
        Ok((&b[..padded_len], self.priv_params.as_ref()))
    }
    fn decrypt<'a: 'c, 'b, 'c>(
        &'a mut self,
        data: &'b [u8],
        usm: &'b UsmParameters<'b>,
    ) -> SnmpResult<ScopedPdu<'c>> {
        // Get IV
        let mut iv = [0u8; 8];
        for (idx, (x, y)) in usm
            .privacy_params
            .iter()
            .zip(self.pre_iv.iter())
            .enumerate()
        {
            iv[idx] = x ^ y;
        }
        //
        let decryptor =
            DesCbcDecryptor::new_from_slices(&self.key, &iv).map_err(|_| SnmpError::InvalidKey)?;
        self.buf.reset();
        self.buf.skip(data.len());
        let b = self.buf.data_mut();
        decryptor
            .decrypt_padded_b2b_mut::<NoPadding>(data, b)
            .map_err(|_| SnmpError::InvalidKey)?;
        // Decode
        let scoped_pdu = ScopedPdu::try_from(self.buf.data())?;
        Ok(scoped_pdu)
    }
}
