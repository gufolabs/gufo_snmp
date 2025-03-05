// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v1 Message
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use crate::ber::{
    BerDecoder, BerEncoder, SnmpInt, SnmpOctetString, SnmpSequence, TAG_INT, TAG_OCTET_STRING,
};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::SNMP_V1;
use crate::snmp::pdu::SnmpPdu;

pub struct SnmpV1Message<'a> {
    pub community: &'a [u8],
    pub pdu: SnmpPdu<'a>,
}

const V1_BER: [u8; 3] = [TAG_INT, 1, SNMP_V1];

impl<'a> TryFrom<&'a [u8]> for SnmpV1Message<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> SnmpResult<SnmpV1Message<'a>> {
        // Top-level sequence
        let (tail, envelope) = SnmpSequence::from_ber(i)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        // Version
        let (tail, v_code) = SnmpInt::from_ber(envelope.0)?;
        let vc = v_code.into();
        if vc != SNMP_V1 {
            return Err(SnmpError::InvalidVersion(vc));
        }
        // Parse community
        let (tail, community) = SnmpOctetString::from_ber(tail)?;
        // Parse PDU
        let pdu = SnmpPdu::try_from(tail)?;
        //
        Ok(SnmpV1Message {
            community: community.0,
            pdu,
        })
    }
}

impl BerEncoder for SnmpV1Message<'_> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        // Push PDU
        self.pdu.push_ber(buf)?;
        // Push community
        buf.push_tagged(TAG_OCTET_STRING, self.community)?;
        // Push version
        buf.push(&V1_BER)?;
        // Push top-level sequence
        buf.push_tag_len(0x30, buf.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ber::SnmpOid;
    use crate::snmp::get::SnmpGet;
    use crate::snmp::value::SnmpValue;

    #[test]
    fn test_parse_snmp_v1_get() -> SnmpResult<()> {
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
        let msg = SnmpV1Message::try_from(data.as_ref())?;
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
    fn test_parse_snmp_getresponse_exception() -> SnmpResult<()> {
        let data = [
            48u8, 40, // Seq 40 bytes
            2, 1, 0, // INTEGER 1, v1
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
        let msg = SnmpV1Message::try_from(data.as_ref())?;
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
    fn test_parse_snmp_getresponse() -> SnmpResult<()> {
        let data = [
            48u8, 55, // Seq 55 bytes
            2, 1, 0, // INTEGER, v1
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
        let msg = SnmpV1Message::try_from(data.as_ref())?;
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
    fn test_parse_snmp_getresponse_many() -> SnmpResult<()> {
        let data = [
            48u8, 129, 134, // Sequence, 134 bytes
            2, 1, 0, // ITEGER, v1
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
        let msg = SnmpV1Message::try_from(data.as_ref())?;
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
    fn test_parse_snmp_getresponse_many_rel() -> SnmpResult<()> {
        let data = [
            48u8, 116, // Sequence, 116 bytes
            2, 1, 0, // ITEGER, v1
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
        let msg = SnmpV1Message::try_from(data.as_ref())?;
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
    fn test_encode_snmp_get() -> SnmpResult<()> {
        let expected = [
            0x30u8, 0x35, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
            0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
            0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
            0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
        ];
        let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
        let msg = SnmpV1Message {
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
