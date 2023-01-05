// ------------------------------------------------------------------------
// Gufo Snmp: BER module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use nom::{Err, IResult};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BerClass {
    Universal,
    Application,
    Context,
    Private,
}

pub(crate) const TAG_BOOL: usize = 0x1;
pub(crate) const TAG_INT: usize = 0x2;
pub(crate) const TAG_OCTET_STRING: usize = 0x4;
pub(crate) const TAG_NULL: usize = 0x5;
pub(crate) const TAG_OBJECT_ID: usize = 0x6;
pub(crate) const TAG_SEQUENCE: usize = 0x10;

pub(crate) mod header;
pub(crate) use header::BerHeader;
pub(crate) mod t_bool;
pub(crate) use t_bool::SnmpBool;
pub(crate) mod t_int;
pub(crate) use t_int::SnmpInt;
pub(crate) mod t_null;
pub(crate) use t_null::SnmpNull;
pub(crate) mod t_objectid;
pub(crate) use t_objectid::SnmpOid;
pub(crate) mod t_octetstring;
pub(crate) use t_octetstring::SnmpOctetString;
pub(crate) mod t_sequence;
pub(crate) use t_sequence::SnmpSequence;
pub(crate) mod t_option;
pub(crate) use t_option::SnmpOption;

use crate::error::SnmpError;

pub(crate) trait BerDecoder<'a>
where
    Self: Sized,
{
    const IS_CONSTRUCTED: bool;
    const TAG: usize;

    fn decode(i: &'a [u8], hdr: &BerHeader) -> Result<Self, SnmpError>;

    fn from_ber(i: &'a [u8]) -> IResult<&'a [u8], Self, SnmpError> {
        if i.len() < 2 {
            return Err(Err::Failure(SnmpError::Incomplete));
        }
        let (tail, hdr) = BerHeader::from_ber(i)?;
        if hdr.constructed != Self::IS_CONSTRUCTED || hdr.tag != Self::TAG {
            return Err(Err::Failure(SnmpError::UnexpectedTag));
        }
        //
        Ok((
            &tail[hdr.length..],
            Self::decode(tail, &hdr).map_err(Err::Failure)?,
        ))
    }
}
