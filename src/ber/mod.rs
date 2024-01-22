// ------------------------------------------------------------------------
// Gufo SNMP: BER module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use nom::{Err, IResult};
use pyo3::{Py, PyAny, Python};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BerClass {
    Universal,
    Application,
    Context,
    Private,
}

pub type Tag = u8;

// BER Tags
// pub const TAG_END_OF_CONTENTS: Tag = 0x0;
pub const TAG_BOOL: Tag = 0x1;
pub const TAG_INT: Tag = 0x2;
// pub const TAG_BIT_STRING: Tag = 0x3;
pub const TAG_OCTET_STRING: Tag = 0x4;
pub const TAG_NULL: Tag = 0x5;
pub const TAG_OBJECT_ID: Tag = 0x6;
pub const TAG_OBJECT_DESCRIPTOR: Tag = 0x7;
pub const TAG_REAL: Tag = 0x9;
pub const TAG_SEQUENCE: Tag = 0x10;
pub const TAG_RELATIVE_OID: Tag = 0xd;
// SNMP Application Tags
pub const TAG_APP_IPADDRESS: Tag = 0;
pub const TAG_APP_COUNTER32: Tag = 1;
pub const TAG_APP_GAUGE32: Tag = 2;
pub const TAG_APP_TIMETICKS: Tag = 3;
pub const TAG_APP_OPAQUE: Tag = 4;
// pub const TAG_APP_NSAPADDRESS: Tag = 5;
pub const TAG_APP_COUNTER64: Tag = 6;
pub const TAG_APP_UINTEGER32: Tag = 7;
// SNMP Context Tags
pub const TAG_CTX_NO_SUCH_OBJECT: Tag = 0;
pub const TAG_CTX_NO_SUCH_INSTANCE: Tag = 1;
pub const TAG_CTX_END_OF_MIB_VIEW: Tag = 2;

pub mod header;
pub use header::BerHeader;
pub mod t_bool;
pub use t_bool::SnmpBool;
pub mod t_int;
pub use t_int::SnmpInt;
pub mod t_null;
pub use t_null::SnmpNull;
pub mod t_objectid;
pub use t_objectid::SnmpOid;
pub mod t_objectdescriptor;
pub use t_objectdescriptor::SnmpObjectDescriptor;
pub mod t_octetstring;
pub use t_octetstring::SnmpOctetString;
pub mod t_real;
pub use t_real::SnmpReal;
pub mod t_relative_oid;
pub use t_relative_oid::SnmpRelativeOid;
pub mod t_sequence;
pub use t_sequence::SnmpSequence;
pub mod t_option;
use crate::buf::Buffer;
pub use t_option::SnmpOption;
pub mod t_ipaddress;
pub use t_ipaddress::SnmpIpAddress;
pub mod t_counter32;
pub use t_counter32::SnmpCounter32;
pub mod t_gauge32;
pub use t_gauge32::SnmpGauge32;
pub mod t_timeticks;
pub use t_timeticks::SnmpTimeTicks;
pub mod t_opaque;
pub use t_opaque::SnmpOpaque;
pub mod t_counter64;
pub use t_counter64::SnmpCounter64;
pub mod t_uinteger32;
pub use t_uinteger32::SnmpUInteger32;

use crate::error::{SnmpError, SnmpResult};

pub trait BerDecoder<'a>
where
    Self: Sized,
{
    const ALLOW_PRIMITIVE: bool;
    const ALLOW_CONSTRUCTED: bool;
    const TAG: Tag;

    fn decode(i: &'a [u8], hdr: &BerHeader) -> SnmpResult<Self>;

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

pub trait BerEncoder {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()>;
}

// Convert value to python under the GIL held
pub trait ToPython {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>>;
}
