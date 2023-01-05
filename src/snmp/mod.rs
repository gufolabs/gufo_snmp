// ------------------------------------------------------------------------
// Gufo Snmp
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

const PDU_GET: usize = 0;
const PDU_GETNEXT: usize = 1;
const PDU_GETRESPONSE: usize = 2;

#[derive(Debug, PartialEq)]
enum SnmpVersion {
    V1,
    V2C,
}

pub(crate) mod get;
pub(crate) mod getresponse;
pub(crate) mod msg;
pub(crate) mod pdu;
pub(crate) mod var;
