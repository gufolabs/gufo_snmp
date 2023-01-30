// ------------------------------------------------------------------------
// Gufo SNMP: BER module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use nom::{Err, IResult};
use pyo3::{Py, PyAny, Python};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BerClass {
    Universal,
    Application,
    Context,
    Private,
}

// BER Tags
// pub(crate) const TAG_END_OF_CONTENTS: usize = 0x0;
pub(crate) const TAG_BOOL: usize = 0x1;
pub(crate) const TAG_INT: usize = 0x2;
// pub(crate) const TAG_BIT_STRING: usize = 0x3;
pub(crate) const TAG_OCTET_STRING: usize = 0x4;
pub(crate) const TAG_NULL: usize = 0x5;
pub(crate) const TAG_OBJECT_ID: usize = 0x6;
pub(crate) const TAG_OBJECT_DESCRIPTOR: usize = 0x7;
pub(crate) const TAG_REAL: usize = 0x9;
pub(crate) const TAG_SEQUENCE: usize = 0x10;
// SNMP Application Tags
pub(crate) const TAG_APP_IPADDRESS: usize = 0;
pub(crate) const TAG_APP_COUNTER32: usize = 1;
pub(crate) const TAG_APP_GAUGE32: usize = 2;
pub(crate) const TAG_APP_TIMETICKS: usize = 3;
pub(crate) const TAG_APP_OPAQUE: usize = 4;
// pub(crate) const TAG_APP_NSAPADDRESS: usize = 5;
pub(crate) const TAG_APP_COUNTER64: usize = 6;
pub(crate) const TAG_APP_UINTEGER32: usize = 7;
// SNMP Context Tags
pub(crate) const TAG_CTX_NO_SUCH_OBJECT: usize = 0;
pub(crate) const TAG_CTX_NO_SUCH_INSTANCE: usize = 1;
pub(crate) const TAG_CTX_END_OF_MIB_VIEW: usize = 2;

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
pub(crate) mod t_objectdescriptor;
pub(crate) use t_objectdescriptor::SnmpObjectDescriptor;
pub(crate) mod t_octetstring;
pub(crate) use t_octetstring::SnmpOctetString;
pub(crate) mod t_real;
pub(crate) use t_real::SnmpReal;
pub(crate) mod t_sequence;
pub(crate) use t_sequence::SnmpSequence;
pub(crate) mod t_option;
use crate::buf::Buffer;
pub(crate) use t_option::SnmpOption;
pub(crate) mod t_ipaddress;
pub(crate) use t_ipaddress::SnmpIpAddress;
pub(crate) mod t_counter32;
pub(crate) use t_counter32::SnmpCounter32;
pub(crate) mod t_gauge32;
pub(crate) use t_gauge32::SnmpGauge32;
pub(crate) mod t_timeticks;
pub(crate) use t_timeticks::SnmpTimeTicks;
pub(crate) mod t_opaque;
pub(crate) use t_opaque::SnmpOpaque;
pub(crate) mod t_counter64;
pub(crate) use t_counter64::SnmpCounter64;
pub(crate) mod t_uinteger32;
pub(crate) use t_uinteger32::SnmpUInteger32;

use crate::error::SnmpError;

pub(crate) trait BerDecoder<'a>
where
    Self: Sized,
{
    const ALLOW_PRIMITIVE: bool;
    const ALLOW_CONSTRUCTED: bool;
    const TAG: usize;

    fn decode(i: &'a [u8], hdr: &BerHeader) -> Result<Self, SnmpError>;

    fn from_ber(i: &'a [u8]) -> IResult<&'a [u8], Self, SnmpError> {
        if i.len() < 2 {
            return Err(Err::Failure(SnmpError::Incomplete));
        }
        let (tail, hdr) = BerHeader::from_ber(i)?;
        if hdr.tag != Self::TAG
            || (hdr.constructed && !Self::ALLOW_CONSTRUCTED)
            || (!hdr.constructed && !Self::ALLOW_PRIMITIVE)
        {
            return Err(Err::Failure(SnmpError::UnexpectedTag));
        }
        //
        Ok((
            &tail[hdr.length..],
            Self::decode(tail, &hdr).map_err(Err::Failure)?,
        ))
    }
}

pub(crate) trait BerEncoder {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError>;
}

// Convert value to python under the GIL held
pub(crate) trait ToPython {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError>;
}
