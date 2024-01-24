// ------------------------------------------------------------------------
// Gufo SNMP: SnmpVar struct
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{
    BerClass, BerDecoder, BerHeader, SnmpBool, SnmpCounter32, SnmpCounter64, SnmpGauge32, SnmpInt,
    SnmpIpAddress, SnmpNull, SnmpObjectDescriptor, SnmpOctetString, SnmpOid, SnmpOpaque, SnmpReal,
    SnmpTimeTicks, SnmpUInteger32, ToPython, TAG_APP_COUNTER32, TAG_APP_COUNTER64, TAG_APP_GAUGE32,
    TAG_APP_IPADDRESS, TAG_APP_OPAQUE, TAG_APP_TIMETICKS, TAG_APP_UINTEGER32, TAG_BOOL,
    TAG_CTX_END_OF_MIB_VIEW, TAG_CTX_NO_SUCH_INSTANCE, TAG_CTX_NO_SUCH_OBJECT, TAG_INT, TAG_NULL,
    TAG_OBJECT_DESCRIPTOR, TAG_OBJECT_ID, TAG_OCTET_STRING, TAG_REAL,
};
use crate::error::{SnmpError, SnmpResult};
use nom::{Err, IResult};
use pyo3::{Py, PyAny, Python};

pub enum SnmpValue<'a> {
    Bool(SnmpBool),
    Int(SnmpInt),
    Null,
    OctetString(SnmpOctetString<'a>),
    Oid(SnmpOid),
    ObjectDescriptor(SnmpObjectDescriptor<'a>),
    Real(SnmpReal),
    IpAddress(SnmpIpAddress),
    Counter32(SnmpCounter32),
    Gauge32(SnmpGauge32),
    TimeTicks(SnmpTimeTicks),
    Opaque(SnmpOpaque<'a>),
    Counter64(SnmpCounter64),
    UInteger32(SnmpUInteger32),
    NoSuchObject,
    NoSuchInstance,
    EndOfMibView,
}

impl<'a> SnmpValue<'a> {
    pub fn from_ber(i: &[u8]) -> IResult<&[u8], SnmpValue, SnmpError> {
        let (tail, hdr) = BerHeader::from_ber(i)?;
        let value = match hdr.constructed {
            // Primitive types
            false => match hdr.class {
                BerClass::Universal => match hdr.tag {
                    // @todo: TAG_END_OF_CONTENTS
                    TAG_BOOL => SnmpValue::Bool(SnmpBool::decode(tail, &hdr)?),
                    TAG_INT => SnmpValue::Int(SnmpInt::decode(tail, &hdr)?),
                    // @todo: TAG_BIT_STRING
                    TAG_OCTET_STRING => {
                        SnmpValue::OctetString(SnmpOctetString::decode(tail, &hdr)?)
                    }
                    TAG_NULL => {
                        SnmpNull::decode(tail, &hdr)?;
                        SnmpValue::Null
                    }
                    TAG_OBJECT_ID => SnmpValue::Oid(SnmpOid::decode(tail, &hdr)?),
                    TAG_OBJECT_DESCRIPTOR => {
                        SnmpValue::ObjectDescriptor(SnmpObjectDescriptor::decode(tail, &hdr)?)
                    }
                    TAG_REAL => SnmpValue::Real(SnmpReal::decode(i, &hdr)?),
                    //
                    _ => {
                        return Err(Err::Failure(SnmpError::UnsupportedTag(format!(
                            "Universal primitive tag {}: {:X?}",
                            hdr.tag, i
                        ))))
                    }
                },
                BerClass::Application => match hdr.tag {
                    TAG_APP_IPADDRESS => SnmpValue::IpAddress(SnmpIpAddress::decode(tail, &hdr)?),
                    TAG_APP_COUNTER32 => SnmpValue::Counter32(SnmpCounter32::decode(tail, &hdr)?),
                    TAG_APP_GAUGE32 => SnmpValue::Gauge32(SnmpGauge32::decode(tail, &hdr)?),
                    TAG_APP_TIMETICKS => SnmpValue::TimeTicks(SnmpTimeTicks::decode(tail, &hdr)?),
                    TAG_APP_OPAQUE => SnmpValue::Opaque(SnmpOpaque::decode(tail, &hdr)?),
                    // TAG_APP_NSAPADDRESS=>{},
                    TAG_APP_COUNTER64 => SnmpValue::Counter64(SnmpCounter64::decode(tail, &hdr)?),
                    TAG_APP_UINTEGER32 => {
                        SnmpValue::UInteger32(SnmpUInteger32::decode(tail, &hdr)?)
                    }
                    _ => {
                        return Err(Err::Failure(SnmpError::UnsupportedTag(format!(
                            "Application primitive tag {}: {:X?}",
                            hdr.tag, i
                        ))))
                    }
                },
                BerClass::Context => match hdr.tag {
                    TAG_CTX_NO_SUCH_OBJECT => SnmpValue::NoSuchObject,
                    TAG_CTX_NO_SUCH_INSTANCE => SnmpValue::NoSuchInstance,
                    TAG_CTX_END_OF_MIB_VIEW => SnmpValue::EndOfMibView,
                    _ => {
                        return Err(Err::Failure(SnmpError::UnsupportedTag(format!(
                            "Context primitive tag {}: {:X?}",
                            hdr.tag, i
                        ))))
                    }
                },
                _ => {
                    return Err(Err::Failure(SnmpError::UnsupportedTag(format!(
                        "{:?} primitive tag {}: {:X?}",
                        hdr.class, hdr.tag, i
                    ))))
                }
            },
            // Constructed types
            true => {
                return Err(Err::Failure(SnmpError::UnsupportedTag(format!(
                    "{:?} constructed tag {}: {:X?}",
                    hdr.class, hdr.tag, i
                ))))
            }
        };
        Ok((&tail[hdr.length..], value))
    }
}

impl<'a> ToPython for &SnmpValue<'a> {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>> {
        Ok(match self {
            SnmpValue::Bool(x) => x.try_to_python(py)?,
            SnmpValue::Int(x) => x.try_to_python(py)?,
            SnmpValue::Null => todo!("None"),
            SnmpValue::OctetString(x) => x.try_to_python(py)?,
            SnmpValue::Oid(x) => x.try_to_python(py)?,
            SnmpValue::ObjectDescriptor(x) => x.try_to_python(py)?,
            SnmpValue::Real(x) => x.try_to_python(py)?,
            SnmpValue::IpAddress(x) => x.try_to_python(py)?,
            SnmpValue::Counter32(x) => x.try_to_python(py)?,
            SnmpValue::Gauge32(x) => x.try_to_python(py)?,
            SnmpValue::TimeTicks(x) => x.try_to_python(py)?,
            SnmpValue::Opaque(x) => x.try_to_python(py)?,
            SnmpValue::Counter64(x) => x.try_to_python(py)?,
            SnmpValue::UInteger32(x) => x.try_to_python(py)?,
            SnmpValue::NoSuchObject | SnmpValue::NoSuchInstance => todo!("never should be passed"),
            SnmpValue::EndOfMibView => todo!("never should be passed"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{SnmpError, SnmpResult};

    #[test]
    fn test_bool() -> SnmpResult<()> {
        let data = [1u8, 1, 1];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Bool(x) = value {
            assert!(bool::from(x));
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }

    #[test]
    fn test_int() -> SnmpResult<()> {
        let data = [2u8, 1, 10];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Int(x) = value {
            let v: i64 = x.into();
            assert_eq!(v, 10);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }

    #[test]
    fn test_null() -> SnmpResult<()> {
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
    fn test_octet_string() -> SnmpResult<()> {
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
    fn test_object_id() -> SnmpResult<()> {
        let data = [0x6u8, 0x8, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x05, 0x00];
        let expected = [1u32, 3, 6, 1, 2, 1, 1, 5, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Oid(x) = value {
            assert_eq!(x.0, &expected);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_object_descriptor() -> SnmpResult<()> {
        let data = [7u8, 5, 0, 1, 2, 3, 4];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::ObjectDescriptor(x) = value {
            assert_eq!(x.0, &data[2..]);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_real() -> SnmpResult<()> {
        let data = [9u8, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Real(x) = value {
            let v: f64 = x.into();
            assert_eq!(v, 0.0);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_ipaddress() -> SnmpResult<()> {
        let data = [0x40, 0x4, 127, 0, 0, 1];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::IpAddress(x) = value {
            let s: String = (&x).into();
            assert_eq!(s, "127.0.0.1");
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_counter32() -> SnmpResult<()> {
        let data = [0x41, 4, 1, 53, 16, 171];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Counter32(x) = value {
            assert_eq!(x.0, 0x013510AB);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_gauge32() -> SnmpResult<()> {
        let data = [0x42, 4, 1, 53, 16, 171];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Gauge32(x) = value {
            assert_eq!(x.0, 0x013510AB);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_timeticks() -> SnmpResult<()> {
        let data = [67, 4, 1, 53, 16, 171];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::TimeTicks(x) = value {
            assert_eq!(x.0, 0x013510AB);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_opaque() -> SnmpResult<()> {
        let data = [0x44, 5, 0, 1, 2, 3, 4];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Opaque(x) = value {
            assert_eq!(x.0, &data[2..]);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_counter64() -> SnmpResult<()> {
        let data = [0x46, 4, 1, 53, 16, 171];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::Counter64(x) = value {
            assert_eq!(x.0, 0x013510AB);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_uinteger32() -> SnmpResult<()> {
        let data = [0x47, 4, 1, 53, 16, 171];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::UInteger32(x) = value {
            assert_eq!(x.0, 0x013510AB);
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_no_such_object() -> SnmpResult<()> {
        let data = [0x80u8, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::NoSuchObject = value {
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_no_such_instance() -> SnmpResult<()> {
        let data = [0x81u8, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::NoSuchInstance = value {
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
    #[test]
    fn test_end_of_mib_view() -> SnmpResult<()> {
        let data = [0x82u8, 0];
        let (tail, value) = SnmpValue::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        if let SnmpValue::EndOfMibView = value {
            Ok(())
        } else {
            Err(SnmpError::UnexpectedTag)
        }
    }
}
