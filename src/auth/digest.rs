// ------------------------------------------------------------------------
// Gufo SNMP: Generic HMAC implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use super::SnmpAuth;
use crate::error::SnmpResult;
use digest::Digest;
use std::marker::PhantomData;

// KS - key size
// SS - signature size
pub struct DigestAuth<D: Digest, const KS: usize, const SS: usize> {
    key: [u8; KS],
    _pd: PhantomData<D>,
}

impl<D: Digest, const KS: usize, const SS: usize> Default for DigestAuth<D, KS, SS> {
    fn default() -> Self {
        let key = [0; KS];
        Self {
            key,
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

impl<D: Digest, const KS: usize, const SS: usize> SnmpAuth for DigestAuth<D, KS, SS> {
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
        let rest_len = PADDED_LENGTH - KS;
        let mut ctx1 = D::new();
        // RFC-3414, pp. 6.3.1. Processing an outgoing message
        // a) extend the authKey to 64 octets by appending 48 zero octets;
        //    save it as extendedAuthKey
        //  >>> Really not necessary
        //  b) obtain IPAD by replicating the octet 0x36 64 times;
        //  >>> need only rest
        //  c) obtain K1 by XORing extendedAuthKey with IPAD;
        //  3) Prepend K1 to the wholeMsg and calculate MD5 digest over it according to [RFC1321].
        //  Instead:
        //  * append xored key
        let k1: Vec<u8> = self.key.iter().map(|&x| x ^ IPAD_VALUE).collect();
        ctx1.update(k1);
        //  * append precalculated rest of IPAD
        ctx1.update(&IPAD_MASK[..rest_len]);
        //  * append whole message
        ctx1.update(&data);
        // get MD5
        let d1 = ctx1.finalize();
        // d) obtain OPAD by replicating the octet 0x5C 64 times;
        // >>> Really not necessary
        // e) obtain K2 by XORing extendedAuthKey with OPAD.
        // 4) Prepend K2 to the result of the step 4 and calculate MD5 digest
        //    over it according to [RFC1321].  Take the first 12 octets of the
        //    final digest - this is Message Authentication Code (MAC).
        // Instead:
        //  * append xored key
        let mut ctx2 = D::new();
        let k2: Vec<u8> = self.key.iter().map(|&x| x ^ OPAD_VALUE).collect();
        ctx2.update(k2);
        //  * append precalculated rest of OPAD
        ctx2.update(&OPAD_MASK[..rest_len]);
        // * append previous digest
        ctx2.update(&d1[..KS]);
        let d2 = ctx2.finalize();
        data[offset..offset + SS].copy_from_slice(&d2[0..SS]);
        Ok(())
    }
}
