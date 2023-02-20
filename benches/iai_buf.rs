// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for Buf functions (Iai)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use gufo_snmp::buf::Buffer;
use iai::black_box;

fn buf_default() {
    Buffer::default();
}

fn buf_push_u8() {
    let mut b = Buffer::default();
    b.push_u8(black_box(10));
}

fn buf_push() {
    let mut b = Buffer::default();
    let chunk = [1u8, 2, 3];
    b.push(&chunk);
}

fn buf_push_ber_len_short() {
    let mut b = Buffer::default();
    b.push_ber_len(1);
}

fn buf_push_ber_len_long() {
    let mut b = Buffer::default();
    b.push_ber_len(128);
}

iai::main!(
    buf_default,
    buf_push_u8,
    buf_push,
    buf_push_ber_len_short,
    buf_push_ber_len_long
);
