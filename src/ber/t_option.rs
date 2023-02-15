// ------------------------------------------------------------------------
// Gufo SNMP: BER Option Class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerClass, BerDecoder, BerHeader, TAG_SEQUENCE};
use crate::error::SnmpError;
use nom::{Err, IResult};

pub struct SnmpOption<'a> {
    pub tag: usize,
    pub value: &'a [u8],
}

impl<'a> BerDecoder<'a> for SnmpOption<'a> {
    const ALLOW_PRIMITIVE: bool = false;
    const ALLOW_CONSTRUCTED: bool = true;
    const TAG: usize = TAG_SEQUENCE;

    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        Ok(SnmpOption {
            tag: h.tag,
            value: &i[..h.length],
        })
    }

    fn from_ber(i: &'a [u8]) -> IResult<&'a [u8], Self, SnmpError> {
        if i.len() < 3 {
            return Err(Err::Failure(SnmpError::Incomplete));
        }
        let (tail, hdr) = BerHeader::from_ber(i)?;
        if !hdr.constructed || hdr.class != BerClass::Context {
            return Err(Err::Failure(SnmpError::UnexpectedTag));
        }
        //
        Ok((
            &tail[hdr.length..],
            Self::decode(tail, &hdr).map_err(Err::Failure)?,
        ))
    }
}
