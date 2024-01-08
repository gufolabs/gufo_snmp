// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Message
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use super::pdu::SnmpPdu;
use super::SnmpVersion;
use crate::ber::{
    BerDecoder, BerEncoder, SnmpInt, SnmpOctetString, SnmpSequence, TAG_OCTET_STRING,
};
use crate::buf::Buffer;
use crate::error::SnmpError;

pub struct SnmpMessage<'a> {
    pub version: SnmpVersion,
    pub community: &'a [u8],
    pub pdu: SnmpPdu<'a>,
}

impl<'a> TryFrom<&'a [u8]> for SnmpMessage<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> Result<SnmpMessage<'a>, SnmpError> {
        // Top-level sequence
        let (tail, envelope) = SnmpSequence::from_ber(i)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        // Version
        let (tail, v_code) = SnmpInt::from_ber(envelope.0)?;
        let vc: u8 = v_code.into();
        let version = vc.try_into()?;
        // Parse community
        let (tail, community) = SnmpOctetString::from_ber(tail)?;
        // Parse PDU
        let pdu = SnmpPdu::try_from(tail)?;
        //
        Ok(SnmpMessage {
            version,
            community: community.0,
            pdu,
        })
    }
}

impl<'a> BerEncoder for SnmpMessage<'a> {
    fn push_ber(&self, buf: &mut Buffer) -> Result<(), SnmpError> {
        // Push PDU
        self.pdu.push_ber(buf)?;
        // Push community
        buf.push(self.community)?;
        buf.push_ber_len(self.community.len())?;
        buf.push_u8(TAG_OCTET_STRING)?;
        // Push version
        self.version.push_ber(buf)?;
        // Push top-level sequence
        buf.push_ber_len(buf.len())?;
        buf.push_u8(0x30)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ber::SnmpOid;
    use crate::snmp::get::SnmpGet;
    use crate::snmp::value::SnmpValue;

    #[test]
    fn test_parse_snmp_v1_get() -> Result<(), SnmpError> {
        let data = [
            0x30u8, 0x35, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
            0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
            0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
            0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
        ];
        let vars: Vec<SnmpOid> = vec![
            SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
            SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V1);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        // Analyze PDU
        match msg.pdu {
            SnmpPdu::GetRequest(pdu) => {
                assert_eq!(pdu.request_id, 0x63ccac7d);
                assert_eq!(pdu.vars, vars);
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }

    #[test]
    fn test_parse_snmp_v2c_get() -> Result<(), SnmpError> {
        let data = [
            0x30u8, 0x35, 0x02, 0x01, 0x01, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
            0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
            0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
            0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
        ];
        let vars: Vec<SnmpOid> = vec![
            SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
            SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        // Analyze PDU
        match msg.pdu {
            SnmpPdu::GetRequest(pdu) => {
                assert_eq!(pdu.request_id, 0x63ccac7d);
                assert_eq!(pdu.vars, vars);
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }

    #[test]
    fn test_parse_snmpv2_getresponse_exception() -> Result<(), SnmpError> {
        let data = [
            48u8, 40, // Seq 40 bytes
            2, 1, 1, // INTEGER 1, v2C
            4, 6, 112, 117, 98, 108, 105, 99, // "public"
            162, 27, // 27 bytes
            2, 4, 94, 189, 217, 172, // Request id, 0x5ebdd9ac
            2, 1, 0, // Error status = 0
            2, 1, 0, // Error index = 0
            48, 13, // Varbinds, 13 bytes
            48, 11, // Var, 11 bytes
            6, 7, 43, 6, 1, 2, 1, 1, 6, // Oid 1.3.6.1.2.1.1.6
            129, 0, // NoSuchObject
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        //
        match msg.pdu {
            SnmpPdu::GetResponse(pdu) => {
                assert_eq!(pdu.request_id, 0x5ebdd9ac);
                assert_eq!(pdu.vars.len(), 1);
                if let SnmpValue::NoSuchInstance = pdu.vars[0].value {
                } else {
                    return Err(SnmpError::InvalidData);
                }
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        //
        Ok(())
    }
    #[test]
    fn test_parse_snmpv2_getresponse() -> Result<(), SnmpError> {
        let data = [
            48u8, 55, // Seq 55 bytes
            2, 1, 1, // INTEGER, v2c
            4, 6, 112, 117, 98, 108, 105, 99, // "public"
            162, 42, // PDU, Get-Response, 42 bytes
            2, 4, 40, 86, 116, 146, // Request id
            2, 1, 0, // error status = 0
            2, 1, 0, // error index = 0
            48, 28, // Varbinds, 28 bytes
            48, 26, // Var item, 26 bytes
            6, 8, 43, 6, 1, 2, 1, 1, 6, 0, // OID 1.3.6.1.2.1.1.6.0
            4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115, 116, // String
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        //
        match msg.pdu {
            SnmpPdu::GetResponse(pdu) => {
                assert_eq!(pdu.request_id, 0x28567492);
                assert_eq!(pdu.error_status, 0);
                assert_eq!(pdu.error_index, 0);
                assert_eq!(pdu.vars.len(), 1);
                assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
                match &pdu.vars[0].value {
                    SnmpValue::OctetString(s) => assert_eq!(
                        s.0,
                        &[71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115, 116]
                    ),
                    _ => return Err(SnmpError::InvalidData),
                }
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }
    #[test]
    fn test_parse_snmpv2_getresponse_many() -> Result<(), SnmpError> {
        let data = [
            48u8, 129, 134, // Sequence, 134 bytes
            2, 1, 1, // ITEGER, v2c
            4, 6, 112, 117, 98, 108, 105, 99, // OCTET STRING, "public"
            162, 121, // PDU, Get-Response, 121 byte
            2, 4, 91, 63, 155, 39, // Request ID, 0x5B3F9B27
            2, 1, 0, // error-status, 0
            2, 1, 0, // error-index, 0
            48, 107, // Varbinds, sequence, 107 bytes
            48, 22, // Var, sequence, 22 bytes
            6, 8, 43, 6, 1, 2, 1, 1, 2, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.2.0
            6, 10, 43, 6, 1, 4, 1, 191, 8, 3, 2,
            10, // OBJECT IDENTIFIER, 1.3.6.1.4.1.1.8072.3.2.10
            48, 16, // Var, sequence, 16  bytes
            6, 8, 43, 6, 1, 2, 1, 1, 3, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.3.0
            67, 4, 1, 53, 16, 171, // TimeTicks, 0x013510AB
            48, 26, // Var, sequennce, 26 bytes
            6, 8, 43, 6, 1, 2, 1, 1, 6, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.6.0
            4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115,
            116, // OCTET STRING
            48, 35, // Var, sequence, 35 bytes
            6, 8, 43, 6, 1, 2, 1, 1, 4, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.4.0
            4, 23, 116, 101, 115, 116, 32, 60, 116, 101, 115, 116, 64, 101, 120, 97, 109, 112, 108,
            101, 46, 99, 111, 109, 62, // OCTET STRING
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        //
        match msg.pdu {
            SnmpPdu::GetResponse(pdu) => {
                assert_eq!(pdu.request_id, 0x5B3F9B27);
                assert_eq!(pdu.error_status, 0);
                assert_eq!(pdu.error_index, 0);
                assert_eq!(pdu.vars.len(), 4);
                // Var 0
                assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.2.0")?);
                match &pdu.vars[0].value {
                    SnmpValue::Oid(s) => {
                        assert_eq!(*s, SnmpOid::try_from("1.3.6.1.4.1.8072.3.2.10")?)
                    }
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 1
                assert_eq!(pdu.vars[1].oid, SnmpOid::try_from("1.3.6.1.2.1.1.3.0")?);
                match &pdu.vars[1].value {
                    SnmpValue::TimeTicks(s) => assert_eq!(s.0, 0x013510AB),
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 2
                assert_eq!(pdu.vars[2].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
                match &pdu.vars[2].value {
                    SnmpValue::OctetString(s) => assert_eq!(s.0, b"Gufo SNMP Test"),
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 3
                assert_eq!(pdu.vars[3].oid, SnmpOid::try_from("1.3.6.1.2.1.1.4.0")?);
                match &pdu.vars[3].value {
                    SnmpValue::OctetString(s) => assert_eq!(s.0, b"test <test@example.com>"),
                    _ => return Err(SnmpError::InvalidData),
                }
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }
    #[test]
    fn test_parse_snmpv2_getresponse_many_rel() -> Result<(), SnmpError> {
        let data = [
            48u8, 116, // Sequence, 116 bytes
            2, 1, 1, // ITEGER, v2c
            4, 6, 112, 117, 98, 108, 105, 99, // OCTET STRING, "public"
            162, 103, // PDU, Get-Response, 103 byte
            2, 4, 91, 63, 155, 39, // Request ID, 0x5B3F9B27
            2, 1, 0, // error-status, 0
            2, 1, 0, // error-index, 0
            48, 89, // Varbinds, sequence, 107 bytes
            48, 22, // Var, sequence, 22 bytes
            6, 8, 43, 6, 1, 2, 1, 1, 2, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.2.0
            6, 10, 43, 6, 1, 4, 1, 191, 8, 3, 2,
            10, // OBJECT IDENTIFIER, 1.3.6.1.4.1.1.8072.3.2.10
            48, 10, // Var, sequence, 10  bytes
            13, 2, 3, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[3.0]
            67, 4, 1, 53, 16, 171, // TimeTicks, 0x013510AB
            48, 20, // Var, sequennce, 20 bytes
            13, 2, 6, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[6.0]
            4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115,
            116, // OCTET STRING
            48, 29, // Var, sequence, 29 bytes
            13, 2, 4, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[4.0]
            4, 23, 116, 101, 115, 116, 32, 60, 116, 101, 115, 116, 64, 101, 120, 97, 109, 112, 108,
            101, 46, 99, 111, 109, 62, // OCTET STRING
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
        //
        match msg.pdu {
            SnmpPdu::GetResponse(pdu) => {
                assert_eq!(pdu.request_id, 0x5B3F9B27);
                assert_eq!(pdu.error_status, 0);
                assert_eq!(pdu.error_index, 0);
                assert_eq!(pdu.vars.len(), 4);
                // Var 0
                assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.2.0")?);
                match &pdu.vars[0].value {
                    SnmpValue::Oid(s) => {
                        assert_eq!(*s, SnmpOid::try_from("1.3.6.1.4.1.8072.3.2.10")?)
                    }
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 1
                assert_eq!(pdu.vars[1].oid, SnmpOid::try_from("1.3.6.1.2.1.1.3.0")?);
                match &pdu.vars[1].value {
                    SnmpValue::TimeTicks(s) => assert_eq!(s.0, 0x013510AB),
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 2
                assert_eq!(pdu.vars[2].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
                match &pdu.vars[2].value {
                    SnmpValue::OctetString(s) => assert_eq!(s.0, b"Gufo SNMP Test"),
                    _ => return Err(SnmpError::InvalidData),
                }
                // Var 3
                assert_eq!(pdu.vars[3].oid, SnmpOid::try_from("1.3.6.1.2.1.1.4.0")?);
                match &pdu.vars[3].value {
                    SnmpValue::OctetString(s) => assert_eq!(s.0, b"test <test@example.com>"),
                    _ => return Err(SnmpError::InvalidData),
                }
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }
    #[test]
    fn test_encode_snmp_v1_get() -> Result<(), SnmpError> {
        let expected = [
            0x30u8, 0x35, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
            0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
            0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
            0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
        ];
        let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
        let msg = SnmpMessage {
            version: SnmpVersion::V1,
            community: &community,
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id: 0x63ccac7d,
                vars: vec![
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
                ],
            }),
        };
        let mut buf = Buffer::default();
        msg.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
    #[test]
    fn test_encode_snmp_v2c_get() -> Result<(), SnmpError> {
        let expected = [
            0x30u8, 0x35, 0x02, 0x01, 0x01, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
            0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
            0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
            0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
        ];
        let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
        let msg = SnmpMessage {
            version: SnmpVersion::V2C,
            community: &community,
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id: 0x63ccac7d,
                vars: vec![
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
                    SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
                ],
            }),
        };
        let mut buf = Buffer::default();
        msg.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
}
