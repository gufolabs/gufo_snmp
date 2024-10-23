// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Messages
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

mod v1;
mod v2c;
pub mod v3;
pub use super::pdu::SnmpPdu;
pub use v1::SnmpV1Message;
pub use v2c::SnmpV2cMessage;
pub use v3::SnmpV3Message;

pub(crate) trait SnmpMessage {
    fn as_pdu(&self) -> &SnmpPdu;
}
