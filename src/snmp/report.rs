// ------------------------------------------------------------------------
// Gufo SNMP: Report PDU Parser
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::error::SnmpError;

pub struct SnmpReport<'a>(pub &'a [u8]);

impl<'a> TryFrom<&'a [u8]> for SnmpReport<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(SnmpReport(value))
    }
}
