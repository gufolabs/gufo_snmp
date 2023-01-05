// ------------------------------------------------------------------------
// Gufo Snmp: SnmpVar struct
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{
    BerClass, BerDecoder, BerHeader, SnmpBool, SnmpInt, SnmpNull, SnmpOctetString, SnmpOid,
    SnmpSequence, TAG_BOOL, TAG_INT, TAG_NULL, TAG_OBJECT_ID, TAG_OCTET_STRING,
};
use crate::error::SnmpError;
use nom::{Err, IResult};

pub(crate) struct SnmpVar<'a> {
    pub(crate) oid: SnmpOid,
    pub(crate) value: SnmpValue<'a>,
}

pub(crate) enum SnmpValue<'a> {
    Bool(SnmpBool),
    Int(SnmpInt),
    Null,
    OctetString(SnmpOctetString<'a>),
    Oid(SnmpOid),
}

impl<'a> SnmpVar<'a> {
    pub(crate) fn from_ber(i: &[u8]) -> IResult<&[u8], SnmpVar, SnmpError> {
        // Parse enclosing sequence
        let (tail, seq) = SnmpSequence::from_ber(i)?;
        // Parse oid
        let (tail, oid) = SnmpOid::from_ber(tail)?;
        // Parse value
        let (tail, value) = SnmpValue::from_ber(tail)?;
        //
        Ok((tail, SnmpVar { oid, value }))
    }
}

impl<'a> SnmpValue<'a> {
    pub(crate) fn from_ber(i: &[u8]) -> IResult<&[u8], SnmpValue, SnmpError> {
        let (tail, hdr) = BerHeader::from_ber(i)?;
        let value = match (hdr.class, hdr.constructed, hdr.tag) {
            // Universal, Primitive
            (BerClass::Universal, false, TAG_BOOL) => {
                SnmpValue::Bool(SnmpBool::decode(tail, &hdr)?)
            }
            (BerClass::Universal, false, TAG_INT) => SnmpValue::Int(SnmpInt::decode(tail, &hdr)?),
            (BerClass::Universal, false, TAG_NULL) => {
                SnmpNull::decode(tail, &hdr)?;
                SnmpValue::Null
            }
            (BerClass::Universal, false, TAG_OCTET_STRING) => {
                SnmpValue::OctetString(SnmpOctetString::decode(tail, &hdr)?)
            }
            (BerClass::Universal, false, TAG_OBJECT_ID) => {
                SnmpValue::Oid(SnmpOid::decode(tail, &hdr)?)
            }
            // Catch all
            _ => return Err(Err::Failure(SnmpError::UnsupportedData)),
        };
        Ok((&tail[hdr.length..], value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SnmpError;

    #[test]
    fn test_bool() -> Result<(), SnmpError> {
        let data = [1u8, 1, 1];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Bool(x) = value {
            assert_eq!(x.as_bool(), true);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }

    #[test]
    fn test_int() -> Result<(), SnmpError> {
        let data = [2u8, 1, 10];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Int(x) = value {
            assert_eq!(x.as_i64(), 10);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }

    #[test]
    fn test_null() -> Result<(), SnmpError> {
        let data = [5u8, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Null = value {
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }

    #[test]
    fn test_octet_string() -> Result<(), SnmpError> {
        let data = [4u8, 5, 0, 1, 2, 3, 4];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::OctetString(x) = value {
            assert_eq!(x.0, &data[2..]);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_object_id() -> Result<(), SnmpError> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [1u64, 3, 6, 1, 2, 1, 1, 5, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Oid(x) = value {
            assert_eq!(x.0, &expected);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
}
