// ------------------------------------------------------------------------
// Gufo SNMP: SNMP PDU
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::get::SnmpGet;
use super::getbulk::SnmpGetBulk;
use super::getresponse::SnmpGetResponse;
use super::report::SnmpReport;
use super::{
    PDU_GET_BULK_REQUEST, PDU_GET_REQUEST, PDU_GET_RESPONSE, PDU_GETNEXT_REQUEST, PDU_REPORT,
};
use crate::ber::{BerDecoder, BerEncoder, SnmpOption};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::reqid::RequestId;

#[allow(clippy::enum_variant_names)]
pub enum SnmpPdu<'a> {
    GetRequest(SnmpGet),
    GetNextRequest(SnmpGet),
    GetResponse(SnmpGetResponse<'a>),
    GetBulkRequest(SnmpGetBulk),
    Report(SnmpReport<'a>),
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
            PDU_REPORT => SnmpPdu::Report(SnmpReport::try_from(opt.value)?),
            _ => return Err(SnmpError::UnknownPdu),
        })
    }
}

impl BerEncoder for SnmpPdu<'_> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        let rest = buf.len();
        match self {
            SnmpPdu::GetRequest(req) => {
                req.push_ber(buf)?;
                buf.push_tag_len(160, buf.len() - rest) // Context + Constructed + PDU_GET_REQUEST(0)
            }
            SnmpPdu::GetNextRequest(req) => {
                req.push_ber(buf)?;
                buf.push_tag_len(161, buf.len() - rest) // Context + Constructed + PDU_GETNEXT_REQUEST(1)
            }
            SnmpPdu::GetBulkRequest(req) => {
                req.push_ber(buf)?;
                buf.push_tag_len(165, buf.len() - rest) // Context + Constructed + PDU_GETBULK_REQUEST(5)
            }
            _ => Err(SnmpError::NotImplemented),
        }
    }
}

impl SnmpPdu<'_> {
    /// Check if request id is match
    pub fn check(&self, request_id: &RequestId) -> bool {
        match self {
            SnmpPdu::GetRequest(pdu) => request_id.check(pdu.request_id),
            SnmpPdu::GetNextRequest(pdu) => request_id.check(pdu.request_id),
            SnmpPdu::GetBulkRequest(pdu) => request_id.check(pdu.request_id),
            SnmpPdu::GetResponse(pdu) => request_id.check(pdu.request_id),
            SnmpPdu::Report(_) => true,
        }
    }
    /// Get GERRESPONSE pdu
    pub fn as_getresponse(&self) -> Option<&SnmpGetResponse<'_>> {
        if let SnmpPdu::GetResponse(pdu) = self {
            Some(pdu)
        } else {
            None
        }
    }
}
