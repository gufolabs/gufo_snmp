// ------------------------------------------------------------------------
// Gufo SNMP: Generic HMAC implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use super::SnmpAuth;
use super::ZEROES;
use crate::error::SnmpResult;
use digest::Digest;
use std::marker::PhantomData;

// KS - key size
// SS - signature size
// BS - HMAC block size
pub struct DigestAuth<D: Digest, const KS: usize, const SS: usize, const BS: usize> {
    key: [u8; KS],
    _pd: PhantomData<D>,
}

impl<D: Digest, const KS: usize, const SS: usize, const BS: usize> Default
    for DigestAuth<D, KS, SS, BS>
{
    fn default() -> Self {
        let key = [0; KS];
        Self {
            key,
            _pd: Default::default(),
        }
    }
}

const IPAD_VALUE: u8 = 0x36;
const OPAD_VALUE: u8 = 0x5c;
const MEGABYTE: usize = 1_048_576;

impl<D: Digest, const KS: usize, const SS: usize, const BS: usize> SnmpAuth
    for DigestAuth<D, KS, SS, BS>
{
    fn as_localized(&mut self, key: &[u8]) {
        self.key.clone_from_slice(key);
    }
    fn as_master(&mut self, key: &[u8], locality: &[u8]) {
        let mut out = [0; KS];
        self.localize(key, locality, &mut out);
        self.key.clone_from_slice(&out);
    }
    fn as_password(&mut self, password: &[u8], locality: &[u8]) {
        let mut master = [0; KS];
        self.password_to_master(password, &mut master);
        self.as_master(&master, locality);
    }
    fn localize(&self, key: &[u8], locality: &[u8], out: &mut [u8]) {
        let mut hasher = D::new();
        hasher.update(key);
        hasher.update(locality);
        hasher.update(key);
        let digest = hasher.finalize();
        out.clone_from_slice(&digest[..out.len()]);
    }
    fn get_key_size(&self) -> usize {
        KS
    }
    fn get_key(&self) -> &[u8] {
        &self.key
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
        out.clone_from_slice(&digest[..KS]);
    }
    fn has_auth(&self) -> bool {
        true
    }
    fn placeholder(&self) -> &'static [u8] {
        &ZEROES[..SS]
    }
    fn sign(&self, data: &mut [u8], offset: usize) -> SnmpResult<()> {
        let mut ipad = [IPAD_VALUE; BS];
        let mut opad = [OPAD_VALUE; BS];
        // RFC-3414, pp. 6.3.1. Processing an outgoing message
        // a) extend the authKey to 64 octets by appending 48 zero octets;
        //    save it as extendedAuthKey
        //    >>> Really not necessary
        // b) obtain IPAD by replicating the octet 0x36 64 times;
        //    >>> need only rest
        // c) obtain K1 by XORing extendedAuthKey with IPAD;
        // 3) Prepend K1 to the wholeMsg and calculate MD5 digest over it according to [RFC1321].
        // Instead:
        // * XOR the key with IPAD
        // d) obtain OPAD by replicating the octet 0x5C 64 times;
        //    >>> Really not necessary
        // e) obtain K2 by XORing extendedAuthKey with OPAD.
        // 4) Prepend K2 to the result of the step 3 and calculate MD5 digest
        //    over it according to [RFC1321].  Take the first 12 octets of the
        //    final digest - this is Message Authentication Code (MAC).
        // Instead:
        // * XOR the key with OPAD
        for i in 0..KS {
            ipad[i] ^= self.key[i];
            opad[i] ^= self.key[i];
        }
        // * append whole message
        let mut ctx1 = D::new();
        ctx1.update(ipad);
        ctx1.update(&data);
        // get digest
        let d1 = ctx1.finalize();
        // * append previous digest
        let mut ctx2 = D::new();
        ctx2.update(opad);
        ctx2.update(d1);
        let d2 = ctx2.finalize();
        data[offset..offset + SS].copy_from_slice(&d2[..SS]);
        Ok(())
    }
}
