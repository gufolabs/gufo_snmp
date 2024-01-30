// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for Buf functions (Iai)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use gufo_snmp::auth::{Md5AuthKey, Sha1AuthKey, SnmpAuth};
use iai::black_box;

fn md5_default() {
    Md5AuthKey::default();
}

const PASSWORD: &[u8; 10] = b"maplesyrup";
const ENGINE_ID: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
const MD5_MASTER_KEY: [u8; 16] = [
    0x9f, 0xaf, 0x32, 0x83, 0x88, 0x4e, 0x92, 0x83, 0x4e, 0xbc, 0x98, 0x47, 0xd8, 0xed, 0xd9, 0x63,
];
const SHA1_MASTER_KEY: [u8; 20] = [
    0x9f, 0xb5, 0xcc, 0x03, 0x81, 0x49, 0x7b, 0x37, 0x93, 0x52, 0x89, 0x39, 0xff, 0x78, 0x8d, 0x5d,
    0x79, 0x14, 0x52, 0x11,
];

fn md5_password_to_master() {
    let key = Md5AuthKey::default();
    let mut out = [0u8; 16];
    key.password_to_master(black_box(PASSWORD), black_box(&mut out));
}

fn md5_localize() {
    let auth = Md5AuthKey::default();
    let mut out = [0u8; 16];
    auth.localize(
        black_box(&MD5_MASTER_KEY),
        black_box(&ENGINE_ID),
        black_box(&mut out),
    );
}

fn sha1_default() {
    Sha1AuthKey::default();
}

fn sha1_password_to_master() {
    let key = Sha1AuthKey::default();
    let mut out = [0u8; 20];
    key.password_to_master(black_box(PASSWORD), black_box(&mut out));
}

fn sha1_localize() {
    let auth = Sha1AuthKey::default();
    let mut out = [0u8; 20];
    auth.localize(
        black_box(&SHA1_MASTER_KEY),
        black_box(&ENGINE_ID),
        black_box(&mut out),
    );
}

iai::main!(
    md5_default,
    md5_password_to_master,
    md5_localize,
    sha1_default,
    sha1_password_to_master,
    sha1_localize,
);
