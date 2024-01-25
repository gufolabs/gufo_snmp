// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for decode functions (Iai)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use gufo_snmp::ber::BerDecoder;
use gufo_snmp::ber::{
    BerHeader, SnmpBool, SnmpCounter32, SnmpCounter64, SnmpGauge32, SnmpInt, SnmpIpAddress,
    SnmpNull, SnmpObjectDescriptor, SnmpOctetString, SnmpOid, SnmpOpaque, SnmpReal,
    SnmpRelativeOid, SnmpTimeTicks, SnmpUInteger32,
};
use gufo_snmp::snmp::msg::SnmpV2cMessage;
use iai::black_box;

pub fn decode_header() {
    let data = [1u8, 1, 0];
    let _ = BerHeader::from_ber(black_box(&data));
}

pub fn decode_getresponse() {
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
    let _ = SnmpV2cMessage::try_from(black_box(data.as_ref()));
}

pub fn decode_bool() {
    let data = [1u8, 1, 0];
    let _ = SnmpBool::from_ber(black_box(&data));
}

pub fn decode_counter32() {
    let data = [0x41, 4, 1, 53, 16, 171];
    let _ = SnmpCounter32::from_ber(black_box(&data));
}

pub fn decode_counter64() {
    let data = [0x46, 4, 1, 53, 16, 171];
    let _ = SnmpCounter64::from_ber(black_box(&data));
}

pub fn decode_gauge32() {
    let data = [0x42, 4, 1, 53, 16, 171];
    let _ = SnmpGauge32::from_ber(black_box(&data));
}

pub fn decode_int() {
    let data = [2, 4, 1, 53, 16, 171];
    let _ = SnmpInt::from_ber(black_box(&data));
}

pub fn decode_ipaddress() {
    let data = [0x40, 0x4, 127, 0, 0, 1];
    let _ = SnmpIpAddress::from_ber(black_box(&data));
}

pub fn decode_null() {
    let data = [5, 0];
    let _ = SnmpNull::from_ber(black_box(&data));
}

pub fn decode_objectdescriptor() {
    let data = [7u8, 5, 0, 1, 2, 3, 4];
    let _ = SnmpObjectDescriptor::from_ber(black_box(&data));
}

pub fn decode_oid() {
    let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
    let _ = SnmpOid::from_ber(black_box(&data));
}

pub fn decode_octetstring() {
    let data = [4u8, 5, 0, 1, 2, 3, 4];
    let _ = SnmpOctetString::from_ber(black_box(&data));
}

pub fn decode_opaque() {
    let data = [0x44, 5, 0, 1, 2, 3, 4];
    let _ = SnmpOpaque::from_ber(black_box(&data));
}

pub fn decode_real_nr1() {
    let data = [9u8, 5, 0x01, 0x2d, 0x34, 0x35, 0x36];
    let _ = SnmpReal::from_ber(black_box(&data));
}

pub fn decode_real_nr2() {
    let data = [9u8, 7, 0x02, 0x2d, 0x34, 0x35, 0x36, 0x2e, 0x37];
    let _ = SnmpReal::from_ber(black_box(&data));
}

pub fn decode_real_nr3() {
    let data = [9u8, 6, 0x03, 0x31, 0x35, 0x45, 0x2d, 0x31];
    let _ = SnmpReal::from_ber(black_box(&data));
}

pub fn decode_relative_oid() {
    let data = [0xd, 4, 0xc2, 0x7b, 0x03, 0x02];
    let _ = SnmpRelativeOid::from_ber(black_box(&data));
}

pub fn decode_timeticks() {
    let data = [0x43, 0x4, 0, 0x89, 0x92, 0xDB];
    let _ = SnmpTimeTicks::from_ber(black_box(&data));
}

pub fn decode_uinteger32() {
    let data = [0x47, 0x4, 0, 0x89, 0x92, 0xDB];
    let _ = SnmpUInteger32::from_ber(black_box(&data));
}

iai::main!(
    decode_header,
    decode_getresponse,
    decode_bool,
    decode_counter32,
    decode_counter64,
    decode_gauge32,
    decode_int,
    decode_ipaddress,
    decode_null,
    decode_objectdescriptor,
    decode_oid,
    decode_octetstring,
    decode_opaque,
    decode_real_nr1,
    decode_real_nr2,
    decode_real_nr3,
    decode_relative_oid,
    decode_timeticks,
    decode_uinteger32,
);
