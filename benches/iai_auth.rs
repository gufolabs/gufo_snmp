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

fn md5_password_to_master() {
    let key = Md5AuthKey::default();
    let mut out = [0u8; 16];
    key.password_to_master(black_box(PASSWORD), black_box(&mut out));
}

fn sha1_default() {
    Sha1AuthKey::default();
}

fn sha1_password_to_master() {
    let key = Sha1AuthKey::default();
    let mut out = [0u8; 20];
    key.password_to_master(black_box(PASSWORD), black_box(&mut out));
}

iai::main!(
    md5_default,
    md5_password_to_master,
    sha1_default,
    sha1_password_to_master
);
