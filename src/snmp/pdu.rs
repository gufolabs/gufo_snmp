// ------------------------------------------------------------------------
// Gufo Snmp: SNMP PDU
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::get::SnmpGet;
use super::getresponse::SnmpGetResponse;
use super::{PDU_GETNEXT_REQUEST, PDU_GET_REQUEST, PDU_GET_RESPONSE};
use crate::ber::{BerDecoder, BerEncoder, SnmpOption};
use crate::buf::Buffer;
use crate::error::SnmpError;

pub(crate) enum SnmpPdu<'a> {
    GetRequest(SnmpGet),
    GetNextRequest(SnmpGet),
    GetResponse(SnmpGetResponse<'a>),
}

impl<'a> TryFrom<&'a [u8]> for SnmpPdu<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let (_, opt) = SnmpOption::from_ber(value)?;
        Ok(match opt.tag {
            PDU_GET_REQUEST => SnmpPdu::GetRequest(SnmpGet::try_from(opt.value)?),
            PDU_GETNEXT_REQUEST => SnmpPdu::GetNextRequest(SnmpGet::try_from(opt.value)?),
            PDU_GET_RESPONSE => SnmpPdu::GetResponse(SnmpGetResponse::try_from(opt.value)?),
            _ => return Err(SnmpError::UnknownPdu),
        })
    }
}

impl<'a> BerEncoder for SnmpPdu<'a> {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
        match self {
            SnmpPdu::GetRequest(req) => {
                req.push_ber(buf)?;
                buf.push_ber_len(buf.len())?;
                buf.push_u8(160)?; // Context + Constructed + PDU_GET_REQUEST(0)
                Ok(())
            }
            _ => Err(SnmpError::NotImplemented),
        }
    }
}
