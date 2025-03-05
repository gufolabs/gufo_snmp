// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for decode functions (Criterion)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use criterion::{Criterion, criterion_group, criterion_main};

use gufo_snmp::ber::BerDecoder;
use gufo_snmp::ber::{
    BerHeader, SnmpBool, SnmpCounter32, SnmpCounter64, SnmpGauge32, SnmpInt, SnmpIpAddress,
    SnmpNull, SnmpObjectDescriptor, SnmpOctetString, SnmpOid, SnmpOpaque, SnmpReal,
    SnmpRelativeOid, SnmpTimeTicks, SnmpUInteger32,
};
use gufo_snmp::snmp::msg::SnmpV2cMessage;

pub fn bench_header(c: &mut Criterion) {
    let data = [1u8, 1, 0];
    c.bench_function("decode header", |b| b.iter(|| BerHeader::from_ber(&data)));
}

pub fn bench_getresponse(c: &mut Criterion) {
    let data = [
        48u8, 129, 134, // Sequence, 134 bytes
        2, 1, 1, // ITEGER, v2c
        4, 6, 112, 117, 98, 108, 105, 99, // OCTET STRING, "public"
        162, 121, // PDU, Get-Response, 121 byte
        2, 4, 91, 63, 155, 39, // Request ID, 0x5B3F9B27
        2, 1, 0, // error-status, 0
        2, 1, 0, // error-index, 0
        48, 107, // Varbinds, sequence, 107 bytes
        48, 22, // Var, sequence, 22 bytes
        6, 8, 43, 6, 1, 2, 1, 1, 2, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.2.0
        6, 10, 43, 6, 1, 4, 1, 191, 8, 3, 2,
        10, // OBJECT IDENTIFIER, 1.3.6.1.4.1.1.8072.3.2.10
        48, 16, // Var, sequence, 16  bytes
        6, 8, 43, 6, 1, 2, 1, 1, 3, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.3.0
        67, 4, 1, 53, 16, 171, // TimeTicks, 0x013510AB
        48, 26, // Var, sequennce, 26 bytes
        6, 8, 43, 6, 1, 2, 1, 1, 6, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.6.0
        4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115, 116, // OCTET STRING
        48, 35, // Var, sequence, 35 bytes
        6, 8, 43, 6, 1, 2, 1, 1, 4, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.4.0
        4, 23, 116, 101, 115, 116, 32, 60, 116, 101, 115, 116, 64, 101, 120, 97, 109, 112, 108,
        101, 46, 99, 111, 109, 62, // OCTET STRING
    ];
    c.bench_function("decode GETRESPONSE", |b| {
        b.iter(|| SnmpV2cMessage::try_from(data.as_ref()))
    });
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
    bench_getresponse,
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
