// ------------------------------------------------------------------------
// Gufo SNMP: SNMP module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::Tag;

const SNMP_V1: u8 = 0;
const SNMP_V2C: u8 = 1;

const PDU_GET_REQUEST: Tag = 0;
const PDU_GETNEXT_REQUEST: Tag = 1;
const PDU_GET_RESPONSE: Tag = 2;
// const PDU_SET_REQUEST: Tag = 3;
// const PDU_TRAP: Tag = 4;
const PDU_GET_BULK_REQUEST: Tag = 5;

pub mod get;
pub mod getbulk;
pub mod getresponse;
pub mod msg;
pub mod pdu;
pub mod value;
