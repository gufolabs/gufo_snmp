// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for decode functions
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use criterion::{criterion_group, criterion_main, Criterion};

use gufo_snmp::ber::BerDecoder;
use gufo_snmp::ber::{
    BerHeader, SnmpBool, SnmpCounter32, SnmpCounter64, SnmpGauge32, SnmpInt, SnmpIpAddress,
    SnmpNull, SnmpObjectDescriptor, SnmpOctetString, SnmpOid, SnmpOpaque, SnmpReal,
    SnmpRelativeOid, SnmpTimeTicks, SnmpUInteger32,
};

pub fn bench_header(c: &mut Criterion) {
    let data = [1u8, 1, 0];
    c.bench_function("decode header", |b| b.iter(|| BerHeader::from_ber(&data)));
}

pub fn bench_bool(c: &mut Criterion) {
    let data = [1u8, 1, 0];
    c.bench_function("decode BOOL", |b| b.iter(|| SnmpBool::from_ber(&data)));
}

pub fn bench_counter32(c: &mut Criterion) {
    let data = [0x41, 4, 1, 53, 16, 171];
    c.bench_function("decode Counter32", |b| {
        b.iter(|| SnmpCounter32::from_ber(&data))
    });
}

pub fn bench_counter64(c: &mut Criterion) {
    let data = [0x46, 4, 1, 53, 16, 171];
    c.bench_function("decode Counter64", |b| {
        b.iter(|| SnmpCounter64::from_ber(&data))
    });
}

pub fn bench_gauge32(c: &mut Criterion) {
    let data = [0x42, 4, 1, 53, 16, 171];
    c.bench_function("decode Gauge32", |b| {
        b.iter(|| SnmpGauge32::from_ber(&data))
    });
}

pub fn bench_int(c: &mut Criterion) {
    let data = [2, 4, 1, 53, 16, 171];
    c.bench_function("decode INTEGER", |b| b.iter(|| SnmpInt::from_ber(&data)));
}

pub fn bench_ipaddress(c: &mut Criterion) {
    let data = [0x40, 0x4, 127, 0, 0, 1];
    c.bench_function("decode IpAddress", |b| {
        b.iter(|| SnmpIpAddress::from_ber(&data))
    });
}

pub fn bench_null(c: &mut Criterion) {
    let data = [5, 0];
    c.bench_function("decode NULL", |b| b.iter(|| SnmpNull::from_ber(&data)));
}

pub fn bench_objectdescriptor(c: &mut Criterion) {
    let data = [7u8, 5, 0, 1, 2, 3, 4];
    c.bench_function("decode OBJECT DESCRIPTOR", |b| {
        b.iter(|| SnmpObjectDescriptor::from_ber(&data))
    });
}

pub fn bench_oid(c: &mut Criterion) {
    let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
    c.bench_function("decode OBJECT IDENTIFIER", |b| {
        b.iter(|| SnmpOid::from_ber(&data))
    });
}

pub fn bench_octetstring(c: &mut Criterion) {
    let data = [4u8, 5, 0, 1, 2, 3, 4];
    c.bench_function("decode OCTET STRING", |b| {
        b.iter(|| SnmpOctetString::from_ber(&data))
    });
}

pub fn bench_opaque(c: &mut Criterion) {
    let data = [0x44, 5, 0, 1, 2, 3, 4];
    c.bench_function("decode Opaque", |b| b.iter(|| SnmpOpaque::from_ber(&data)));
}

pub fn bench_real_nr1(c: &mut Criterion) {
    let data = [9u8, 5, 0x01, 0x2d, 0x34, 0x35, 0x36];
    c.bench_function("decode REAL NR1", |b| b.iter(|| SnmpReal::from_ber(&data)));
}

pub fn bench_real_nr2(c: &mut Criterion) {
    let data = [9u8, 7, 0x02, 0x2d, 0x34, 0x35, 0x36, 0x2e, 0x37];
    c.bench_function("decode REAL NR2", |b| b.iter(|| SnmpReal::from_ber(&data)));
}

pub fn bench_real_nr3(c: &mut Criterion) {
    let data = [9u8, 6, 0x03, 0x31, 0x35, 0x45, 0x2d, 0x31];
    c.bench_function("decode REAL NR3", |b| b.iter(|| SnmpReal::from_ber(&data)));
}

pub fn bench_relative_oid(c: &mut Criterion) {
    let data = [0xd, 4, 0xc2, 0x7b, 0x03, 0x02];
    c.bench_function("decode RELATIVE OBJECT IDENTIFIER", |b| {
        b.iter(|| SnmpRelativeOid::from_ber(&data))
    });
}

pub fn bench_timeticks(c: &mut Criterion) {
    let data = [0x43, 0x4, 0, 0x89, 0x92, 0xDB];
    c.bench_function("decode TimeTicks", |b| {
        b.iter(|| SnmpTimeTicks::from_ber(&data))
    });
}

pub fn bench_uinteger32(c: &mut Criterion) {
    let data = [0x47, 0x4, 0, 0x89, 0x92, 0xDB];
    c.bench_function("decode UInteger32", |b| {
        b.iter(|| SnmpUInteger32::from_ber(&data))
    });
}

criterion_group!(
    benches,
    bench_header,
    bench_bool,
    bench_counter32,
    bench_counter64,
    bench_gauge32,
    bench_int,
    bench_ipaddress,
    bench_null,
    bench_objectdescriptor,
    bench_oid,
    bench_octetstring,
    bench_opaque,
    bench_real_nr1,
    bench_real_nr2,
    bench_real_nr3,
    bench_relative_oid,
    bench_timeticks,
    bench_uinteger32,
);
criterion_main!(benches);
