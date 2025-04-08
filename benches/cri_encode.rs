// ------------------------------------------------------------------------
// Gufo SNMP: Benchmarks for encode functions (Criterion)
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use criterion::{Criterion, criterion_group, criterion_main};
use gufo_snmp::{
    ber::{BerEncoder, SnmpOid},
    buf::Buffer,
    snmp::get::SnmpGet,
    snmp::msg::SnmpV2cMessage,
    snmp::pdu::SnmpPdu,
};

pub fn bench_get(c: &mut Criterion) {
    let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
    let msg = SnmpV2cMessage {
        community: &community,
        pdu: SnmpPdu::GetRequest(SnmpGet {
            request_id: 0x63ccac7d,
            vars: vec![
                SnmpOid::try_from("1.3.6.1.2.1.1.3"),
                SnmpOid::try_from("1.3.6.1.2.1.1.2"),
                SnmpOid::try_from("1.3.6.1.2.1.1.6"),
                SnmpOid::try_from("1.3.6.1.2.1.1.4"),
            ],
        }),
    };
    let mut buf = Buffer::default();

    c.bench_function("encode GET", |b| {
        b.iter(|| {
            buf.reset();
            let _ = msg.push_ber(&mut buf);
        })
    });
}

criterion_group!(benches, bench_get);
criterion_main!(benches);
