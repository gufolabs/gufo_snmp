// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for Buf functions (Iai)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use gufo_snmp::buf::Buffer;
use iai::black_box;

fn buf_default() {
    black_box(Buffer::default());
}

fn buf_push_u8() {
    let mut b = Buffer::default();
    let _ = b.push_u8(black_box(10));
}

fn buf_push() {
    let mut b = Buffer::default();
    let chunk = [1u8, 2, 3];
    let _ = b.push(&chunk);
}

fn buf_push_tag_len_short() {
    let mut b = Buffer::default();
    let _ = b.push_tag_len(4, 1);
}

fn buf_push_tag_len_long1() {
    let mut b = Buffer::default();
    let _ = b.push_tag_len(4, 128);
}
fn buf_push_tag_len_long2() {
    let mut b = Buffer::default();
    let _ = b.push_tag_len(4, 256);
}

iai::main!(
    buf_default,
    buf_push_u8,
    buf_push,
    buf_push_tag_len_short,
    buf_push_tag_len_long1,
    buf_push_tag_len_long2
);
