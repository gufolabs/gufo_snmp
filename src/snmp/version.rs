// ------------------------------------------------------------------------
// Gufo Snmp: SnmpVersion
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{SNMP_V1, SNMP_V2C};
use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::SnmpError;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SnmpVersion {
    V1,
    V2C,
}

impl TryInto<SnmpVersion> for u8 {
    type Error = SnmpError;

    fn try_into(self) -> Result<SnmpVersion, Self::Error> {
        match self {
            SNMP_V1 => Ok(SnmpVersion::V1),
            SNMP_V2C => Ok(SnmpVersion::V2C),
            _ => Err(SnmpError::InvalidVersion(self)),
        }
    }
}

const V1_BER: [u8; 3] = [2, 1, 0];
const V2C_BER: [u8; 3] = [2, 1, 1];

impl BerEncoder for SnmpVersion {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
        match self {
            SnmpVersion::V1 => buf.push(&V1_BER)?,
            SnmpVersion::V2C => buf.push(&V2C_BER)?,
        }
        Ok(())
    }
}
