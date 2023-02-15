// ------------------------------------------------------------------------
// Gufo SNMP: SNMP module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

const SNMP_V1: u8 = 0;
const SNMP_V2C: u8 = 1;

const PDU_GET_REQUEST: usize = 0;
const PDU_GETNEXT_REQUEST: usize = 1;
const PDU_GET_RESPONSE: usize = 2;
// const PDU_SET_REQUEST: usize = 3;
// const PDU_TRAP: usize = 4;
const PDU_GET_BULK_REQUEST: usize = 5;

pub mod get;
pub mod getbulk;
pub mod getresponse;
pub mod msg;
pub mod pdu;
pub mod value;
pub mod version;
pub use version::SnmpVersion;
