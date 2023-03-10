// ------------------------------------------------------------------------
// Gufo SNMP: SNMP PDU
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::get::SnmpGet;
use super::getbulk::SnmpGetBulk;
use super::getresponse::SnmpGetResponse;
use super::{PDU_GETNEXT_REQUEST, PDU_GET_BULK_REQUEST, PDU_GET_REQUEST, PDU_GET_RESPONSE};
use crate::ber::{BerDecoder, BerEncoder, SnmpOption};
use crate::buf::Buffer;
use crate::error::SnmpError;

#[allow(clippy::enum_variant_names)]
pub enum SnmpPdu<'a> {
    GetRequest(SnmpGet),
    GetNextRequest(SnmpGet),
    GetResponse(SnmpGetResponse<'a>),
    GetBulkRequest(SnmpGetBulk),
}

impl<'a> TryFrom<&'a [u8]> for SnmpPdu<'a> {
    type Error = SnmpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let (_, opt) = SnmpOption::from_ber(value)?;
        Ok(match opt.tag {
            PDU_GET_REQUEST => SnmpPdu::GetRequest(SnmpGet::try_from(opt.value)?),
            PDU_GETNEXT_REQUEST => SnmpPdu::GetNextRequest(SnmpGet::try_from(opt.value)?),
            PDU_GET_RESPONSE => SnmpPdu::GetResponse(SnmpGetResponse::try_from(opt.value)?),
            PDU_GET_BULK_REQUEST => SnmpPdu::GetBulkRequest(SnmpGetBulk::try_from(opt.value)?),
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
            SnmpPdu::GetNextRequest(req) => {
                req.push_ber(buf)?;
                buf.push_ber_len(buf.len())?;
                buf.push_u8(161)?; // Context + Constructed + PDU_GETNEXT_REQUEST(1)
                Ok(())
            }
            SnmpPdu::GetBulkRequest(req) => {
                req.push_ber(buf)?;
                buf.push_ber_len(buf.len())?;
                buf.push_u8(165)?; // Context + Constructed + PDU_GETBULK_REQUEST(5)
                Ok(())
            }
            _ => Err(SnmpError::NotImplemented),
        }
    }
}
