// ------------------------------------------------------------------------
// Gufo Snmp: SNMP PDU
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::get::SnmpGet;
use super::getresponse::SnmpGetResponse;
use super::{PDU_GET, PDU_GETNEXT, PDU_GETRESPONSE};
use crate::ber::{BerDecoder, SnmpOption};
use crate::error::SnmpError;

pub(crate) enum SnmpPdu<'a> {
    Get(SnmpGet),
    GetNext(SnmpGet),
    GetResponse(SnmpGetResponse<'a>),
}

impl<'a> TryFrom<&'a [u8]> for SnmpPdu<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let (_, opt) = SnmpOption::from_ber(value)?;
        Ok(match opt.tag {
            PDU_GET => SnmpPdu::Get(SnmpGet::try_from(opt.value)?),
            PDU_GETNEXT => SnmpPdu::GetNext(SnmpGet::try_from(opt.value)?),
            PDU_GETRESPONSE => SnmpPdu::GetResponse(SnmpGetResponse::try_from(opt.value)?),
            _ => return Err(SnmpError::UnknownPdu),
        })
    }
}
