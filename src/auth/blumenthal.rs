// ------------------------------------------------------------------------
// Gufo SNMP: Blumenthal key expansion
// ------------------------------------------------------------------------
// Copyright (C) 2026, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::SnmpAuth;
use crate::error::SnmpResult;
use digest::Digest;
use std::marker::PhantomData;

// KS  - authentication key size
// SS  - signature size
// PKS - expanded privacy key size
pub struct DigestAuthWithBlumenthal<D: Digest, const KS: usize, const SS: usize, const PKS: usize> {
    key: [u8; KS],
    expanded_key: [u8; PKS],
    _pd: PhantomData<D>,
}

impl<D: Digest, const KS: usize, const SS: usize, const PKS: usize> Default
    for DigestAuthWithBlumenthal<D, KS, SS, PKS>
{
    fn default() -> Self {
        Self {
            key: [0; KS],
            expanded_key: [0; PKS],
            _pd: Default::default(),
        }
    }
}

const PADDED_LENGTH: usize = 64;
const ZEROES: [u8; PADDED_LENGTH] = [0; PADDED_LENGTH];
const IPAD_VALUE: u8 = 0x36;
const OPAD_VALUE: u8 = 0x5c;
const IPAD_MASK: [u8; PADDED_LENGTH] = [IPAD_VALUE; PADDED_LENGTH];
const OPAD_MASK: [u8; PADDED_LENGTH] = [OPAD_VALUE; PADDED_LENGTH];
const MEGABYTE: usize = 1_048_576;

impl<D: Digest, const KS: usize, const SS: usize, const PKS: usize>
    DigestAuthWithBlumenthal<D, KS, SS, PKS>
{
    fn expand(&mut self) {
        self.expanded_key[..KS].copy_from_slice(&self.key);
        let mut current = self.key.to_vec();
        let mut offset = KS;
        while offset < PKS {
            let mut hasher = D::new();
            hasher.update(&current);
            let digest = hasher.finalize();
            let n = (PKS - offset).min(digest.len());
            self.expanded_key[offset..offset + n].copy_from_slice(&digest[..n]);
            offset += n;
            current.extend_from_slice(&digest);
        }
    }

    fn localize(&self, key: &[u8], locality: &[u8], out: &mut [u8]) {
        let mut hasher = D::new();
        hasher.update(key);
        hasher.update(locality);
        hasher.update(key);
        let digest = hasher.finalize();
        out.copy_from_slice(&digest[..out.len()]);
    }

    fn password_to_master(&self, password: &[u8], out: &mut [u8]) {
        let mut hasher = D::new();
        let pass_len = password.len();
        let n = MEGABYTE / pass_len;
        let rem = MEGABYTE % pass_len;
        for _ in 0..n {
            hasher.update(password);
        }
        if rem > 0 {
            hasher.update(&password[..rem]);
        }
        let digest = hasher.finalize();
        out.copy_from_slice(&digest[..out.len()]);
    }

    fn sign_with_key(&self, key: &[u8], data: &mut [u8], offset: usize) -> SnmpResult<()> {
        let rest_len = PADDED_LENGTH - KS;
        let mut ctx1 = D::new();
        let k1: Vec<u8> = key.iter().map(|&x| x ^ IPAD_VALUE).collect();
        ctx1.update(k1);
        ctx1.update(&IPAD_MASK[..rest_len]);
        ctx1.update(&*data);
        let d1 = ctx1.finalize();
        let mut ctx2 = D::new();
        let k2: Vec<u8> = key.iter().map(|&x| x ^ OPAD_VALUE).collect();
        ctx2.update(k2);
        ctx2.update(&OPAD_MASK[..rest_len]);
        ctx2.update(&d1[..KS]);
        let d2 = ctx2.finalize();
        data[offset..offset + SS].copy_from_slice(&d2[..SS]);
        Ok(())
    }
}

impl<D: Digest, const KS: usize, const SS: usize, const PKS: usize> SnmpAuth
    for DigestAuthWithBlumenthal<D, KS, SS, PKS>
{
    fn as_localized(&mut self, key: &[u8]) {
        self.key.copy_from_slice(&key[..KS]);
        self.expand();
    }

    fn as_master(&mut self, key: &[u8], locality: &[u8]) {
        let mut out = [0; KS];
        self.localize(key, locality, &mut out);
        self.key.copy_from_slice(&out);
        self.expand();
    }

    fn as_password(&mut self, password: &[u8], locality: &[u8]) {
        let mut master = [0; KS];
        self.password_to_master(password, &mut master);
        self.as_master(&master, locality);
    }

    fn localize(&self, key: &[u8], locality: &[u8], out: &mut [u8]) {
        self.localize(key, locality, out);
    }

    fn get_key_size(&self) -> usize {
        PKS
    }

    fn get_key(&self) -> &[u8] {
        &self.expanded_key
    }

    fn password_to_master(&self, password: &[u8], out: &mut [u8]) {
        self.password_to_master(password, out);
    }

    fn has_auth(&self) -> bool {
        true
    }

    fn placeholder(&self) -> &'static [u8] {
        &ZEROES[..SS]
    }

    fn sign(&self, data: &mut [u8], offset: usize) -> SnmpResult<()> {
        self.sign_with_key(&self.key, data, offset)
    }
}
