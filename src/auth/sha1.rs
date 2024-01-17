// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 Sha1 Auth
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::SnmpAuth;
use sha1::Digest;

pub struct Sha1(Vec<u8>);

impl Sha1 {
    pub fn new(key: Vec<u8>) -> Sha1 {
        Sha1(key)
    }
}

const KEY_LENGTH: usize = 20;
const PADDED_LENGTH: usize = 64;
const REST_LENGTH: usize = PADDED_LENGTH - KEY_LENGTH;
const IPAD_VALUE: u8 = 0x36;
const OPAD_VALUE: u8 = 0x5c;
const IPAD_REST: [u8; REST_LENGTH] = [IPAD_VALUE; REST_LENGTH];
const OPAD_REST: [u8; REST_LENGTH] = [OPAD_VALUE; REST_LENGTH];
const SIGN_SIZE: usize = 12;

impl SnmpAuth for Sha1 {
    fn localize(&mut self, engine_id: &[u8]) {
        let mut result = Vec::with_capacity(2 * self.0.len() + engine_id.len());
        result.extend_from_slice(self.0.as_ref());
        result.extend_from_slice(engine_id.as_ref());
        result.extend_from_slice(self.0.as_ref());
        let mut hasher = sha1::Sha1::new();
        hasher.update(result);
        let digest: [u8; KEY_LENGTH] = hasher.finalize().into();
        self.0.resize(KEY_LENGTH, 0);
        self.0.clone_from_slice(&digest);
    }
    fn sign(&self, whole_msg: &mut [u8], offset: usize) {
        let mut ctx1 = sha1::Sha1::new();
        // RFC-3414, pp. 7.3.1. Processing an outgoing message
        // a) extend the authKey to 64 octets by appending 44 zero octets;
        //    save it as extendedAuthKey
        //  >>> Really not necessary
        //  b) obtain IPAD by replicating the octet 0x36 64 times;
        //  >>> need only rest
        //  c) obtain K1 by XORing extendedAuthKey with IPAD;
        //  3) Prepend K1 to the wholeMsg and calculate MD5 digest over it according to [RFC1321].
        //  Instead:
        //  * append xored key
        let k1: Vec<u8> = self.0.iter().map(|&x| x ^ IPAD_VALUE).collect();
        ctx1.update(k1);
        //  * append precalculated rest of IPAD
        ctx1.update(IPAD_REST);
        //  * append whole message
        ctx1.update(&whole_msg);
        // get Sha1
        let d1: [u8; KEY_LENGTH] = ctx1.finalize().into();
        // d) obtain OPAD by replicating the octet 0x5C 64 times;
        // >>> Really not necessary
        // e) obtain K2 by XORing extendedAuthKey with OPAD.
        // 4) Prepend K2 to the result of the step 4 and calculate MD5 digest
        //    over it according to [RFC1321].  Take the first 12 octets of the
        //    final digest - this is Message Authentication Code (MAC).
        // Instead:
        //  * append xored key
        let mut ctx2 = sha1::Sha1::new();
        let k2: Vec<u8> = self.0.iter().map(|&x| x ^ OPAD_VALUE).collect();
        ctx2.update(k2);
        //  * append precalculated rest of OPAD
        ctx2.update(OPAD_REST);
        // * append previous digest
        ctx2.update(d1);
        let d2: [u8; KEY_LENGTH] = ctx2.finalize().into(); // use only 12 octets
        whole_msg[offset..offset + SIGN_SIZE].copy_from_slice(&d2[0..SIGN_SIZE]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test() -> SnmpResult<()> {
    //     let mut whole_msg = [
    //         48u8, 119, 2, 1, 3, 48, 16, 2, 4, 31, 120, 150, 153, 2, 2, 5, 220, 4, 1, 1, 2, 1, 3, 4,
    //         47, 48, 45, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 2, 1, 0, 2,
    //         1, 0, 4, 6, 117, 115, 101, 114, 49, 48, 4, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
    //         0, 48, 47, 4, 13, 128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100, 4, 0, 160, 28,
    //         2, 4, 80, 85, 225, 64, 2, 1, 0, 2, 1, 0, 48, 14, 48, 12, 6, 8, 43, 6, 1, 2, 1, 1, 4, 0,
    //         5, 0,
    //     ];
    //     let expected = [18, 138, 173, 156, 223, 188, 26, 178, 137, 113, 25, 22];
    //     let master_key = vec![117u8, 115, 101, 114, 49, 48, 107, 101, 121]; // user10key
    //     let engine_id = [128, 0, 31, 136, 4, 50, 55, 103, 83, 56, 54, 116, 100];
    //     let mut key = Md5::new(master_key);
    //     key.localize(&engine_id);
    //     let r = key.get_signature(&whole_msg);
    //     assert_eq!(r, expected);
    //     Ok(())
    // }
}
