// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for encode functions (Iai)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use gufo_snmp::{
    ber::{BerEncoder, SnmpOid},
    buf::Buffer,
    snmp::get::SnmpGet,
    snmp::msg::SnmpV2cMessage,
    snmp::pdu::SnmpPdu,
};
use iai::black_box;

pub fn encode_get() {
    let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
    let msg = SnmpV2cMessage {
        community: &community,
        pdu: SnmpPdu::GetRequest(SnmpGet {
            request_id: 0x63ccac7d,
            vars: vec![
                SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
                SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
                SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 6]),
                SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 4]),
            ],
        }),
    };
    let mut buf = Buffer::default();
    let _ = msg.push_ber(black_box(&mut buf));
}

iai::main!(encode_get);
