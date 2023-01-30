// ------------------------------------------------------------------------
// Gufo SNMP: SNMP message encoding/decoding benchmarks
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use criterion::{criterion_group, criterion_main, Criterion};
use gufo_snmp::{
    ber::SnmpOid,
    buf::Buffer,
    snmp::{SnmpGet, SnmpMessage, SnmpPdu, SnmpVersion},
};

fn msg_benchmark(c: &mut Criterion) {
    c.bench_function("snmp_v2c_get_encode", |b| {
        let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
        let msg = SnmpMessage {
            version: SnmpVersion::V2C,
            community: &community,
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id: 0x63ccac7d,
                vars: vec![
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
                ],
            }),
        };
        let mut b = Buffer::default();
        b.iter(|| {
            b.reset();
            msg.push_ber(&mut b);
        })
    });
}

criterion_group!(benches, msg_benchmark);
criterion_main!(benches);
