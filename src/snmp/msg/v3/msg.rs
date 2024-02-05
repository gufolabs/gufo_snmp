// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 Message
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use super::data::MsgData;
use super::usm::UsmParameters;
use crate::ber::{
    BerDecoder, BerEncoder, SnmpInt, SnmpOctetString, SnmpSequence, TAG_INT, TAG_OCTET_STRING,
};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use crate::snmp::SNMP_V3;

pub struct SnmpV3Message<'a> {
    pub msg_id: i64,
    //pub context_engine_id: &'a [u8],
    //pub context_engine_name: &'a [u8],
    pub flag_auth: bool,
    pub flag_priv: bool,
    pub flag_report: bool,
    pub usm: UsmParameters<'a>,
    pub data: MsgData<'a>,
}

const V3_BER: [u8; 3] = [TAG_INT, 1, SNMP_V3];
const MAX_SIZE: i64 = 2048;
const USM: u8 = 3;
const USM_MODEL_BER: [u8; 3] = [TAG_INT, 1, USM];
// Flags
const FLAG_REPORT: u8 = 4;
const FLAG_PRIV: u8 = 2;
const FLAG_AUTH: u8 = 1;

impl<'a> TryFrom<&'a [u8]> for SnmpV3Message<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> SnmpResult<SnmpV3Message<'a>> {
        // Top-level sequence
        let (tail, envelope) = SnmpSequence::from_ber(i)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        // Version
        let (tail, v_code) = SnmpInt::from_ber(envelope.0)?;
        let vc = v_code.into();
        if vc != SNMP_V3 {
            return Err(SnmpError::InvalidVersion(vc));
        }
        //
        // Parse global header
        //
        let (sp_tail, envelope) = SnmpSequence::from_ber(tail)?;
        // msg id
        let (tail, msg_id_data) = SnmpInt::from_ber(envelope.0)?;
        // max_size
        let (tail, _max_size) = SnmpInt::from_ber(tail)?;
        // flags
        let (tail, flags_data) = SnmpOctetString::from_ber(tail)?;
        if flags_data.0.len() != 1 {
            return Err(SnmpError::InvalidPdu);
        }
        let flags = flags_data.0[0];
        // security model
        let (_, security_model) = SnmpInt::from_ber(tail)?;
        let sm: u8 = security_model.into();
        if sm != USM {
            return Err(SnmpError::UnknownSecurityModel);
        }
        //
        // Process security parameters
        //
        let (tail, security_parameters) = SnmpOctetString::from_ber(sp_tail)?;
        let usm = UsmParameters::try_from(security_parameters.0)?;
        Ok(SnmpV3Message {
            msg_id: msg_id_data.into(),
            flag_auth: (flags & FLAG_AUTH) != 0,
            flag_priv: (flags & FLAG_PRIV) != 0,
            flag_report: (flags & FLAG_REPORT) != 0,
            usm,
            data: MsgData::try_from(tail)?,
        })
    }
}

impl<'a> BerEncoder for SnmpV3Message<'a> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        //
        // Scoped PDU
        //
        self.data.push_ber(buf)?;
        //
        // Push security parameters
        //
        let ln = buf.len();
        self.usm.push_ber(buf)?;
        // Wrap in octet-stream
        buf.push_tag_len(TAG_OCTET_STRING, buf.len() - ln)?;
        //
        // Push global header
        //
        let ln = buf.len();
        // Push security model
        buf.push(&USM_MODEL_BER)?;
        // Push flags
        let mut flag = 0u8;
        if self.flag_auth {
            flag |= FLAG_AUTH
        }
        if self.flag_priv {
            flag |= FLAG_PRIV
        }
        if self.flag_report {
            flag |= FLAG_REPORT;
        }
        buf.push_u8(flag)?;
        buf.push_tag_len(TAG_OCTET_STRING, 1)?;
        // Push msg max size
        let ms: SnmpInt = MAX_SIZE.into();
        ms.push_ber(buf)?;
        // Push msg id
        let msg_id: SnmpInt = self.msg_id.into();
        msg_id.push_ber(buf)?;
        // Push sequece header
        buf.push_tag_len(0x30, buf.len() - ln)?;
        // Push version
        buf.push(&V3_BER)?;
        // Push top-level sequence
        buf.push_tag_len(0x30, buf.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ber::SnmpOid;
    use crate::snmp::get::SnmpGet;
    use crate::snmp::msg::v3::ScopedPdu;
    use crate::snmp::pdu::SnmpPdu;

    #[test]
    fn test_parse_snmp_get() -> SnmpResult<()> {
        let data = [
            0x30u8, 0x40, 0x02, 0x01, 0x03, 0x30, 0x0f, 0x02, 0x03, 0x00, 0x91, 0xc8, 0x02, 0x02,
            0x05, 0xdc, 0x04, 0x01, 0x00, 0x02, 0x01, 0x03, 0x04, 0x15, 0x30, 0x13, 0x04, 0x00,
            0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x04, 0x05, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x04,
            0x00, 0x04, 0x00, 0x30, 0x13, 0x04, 0x00, 0x04, 0x00, 0xa0, 0x0d, 0x02, 0x03, 0x00,
            0x91, 0xc8, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x00,
        ];
        let vars: Vec<SnmpOid> = vec![];
        let msg = SnmpV3Message::try_from(data.as_ref())?;
        // Analyze global header
        assert_eq!(msg.msg_id, 37320);
        assert!(!msg.flag_auth);
        assert!(!msg.flag_priv);
        // Analyze security parameters
        assert_eq!(msg.usm.user_name, "admin".as_bytes());
        // Analyze scoped pdu
        assert_eq!(msg.usm.engine_id.len(), 0);
        //assert_eq!(msg.context_engine_name, vec![]);
        // Analyze PDU
        match msg.data {
            MsgData::Plaintext(scoped) => match scoped.pdu {
                SnmpPdu::GetRequest(pdu) => {
                    assert_eq!(pdu.request_id, 37320);
                    assert_eq!(pdu.vars, vars);
                }
                _ => return Err(SnmpError::InvalidPdu),
            },
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }

    #[test]
    fn test_encode_snmp_get() -> SnmpResult<()> {
        let expected = [
            0x30u8, 0x40, 0x02, 0x01, 0x03, 0x30, 0x0f, 0x02, 0x03, 0x00, 0x91, 0xc8, 0x02, 0x02,
            0x08, 0x00, 0x04, 0x01, 0x00, 0x02, 0x01, 0x03, 0x04, 0x15, 0x30, 0x13, 0x04, 0x00,
            0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x04, 0x05, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x04,
            0x00, 0x04, 0x00, 0x30, 0x13, 0x04, 0x00, 0x04, 0x00, 0xa0, 0x0d, 0x02, 0x03, 0x00,
            0x91, 0xc8, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x00,
        ];
        let empty: [u8; 0] = [];
        let msg = SnmpV3Message {
            msg_id: 37320,
            flag_auth: false,
            flag_priv: false,
            flag_report: false,
            usm: UsmParameters {
                engine_id: &empty,
                engine_boots: 0,
                engine_time: 0,
                user_name: "admin".as_bytes(),
                auth_params: &empty,
                privacy_params: &empty,
            },
            data: MsgData::Plaintext(ScopedPdu {
                engine_id: &empty,
                pdu: SnmpPdu::GetRequest(SnmpGet {
                    request_id: 37320,
                    vars: vec![],
                }),
            }),
        };
        let mut buf = Buffer::default();
        msg.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }

    // #[test]
    // fn test_parse_snmp_getresponse_exception() -> SnmpResult<()> {
    //     let data = [
    //         48u8, 40, // Seq 40 bytes
    //         2, 1, 0, // INTEGER 1, v1
    //         4, 6, 112, 117, 98, 108, 105, 99, // "public"
    //         162, 27, // 27 bytes
    //         2, 4, 94, 189, 217, 172, // Request id, 0x5ebdd9ac
    //         2, 1, 0, // Error status = 0
    //         2, 1, 0, // Error index = 0
    //         48, 13, // Varbinds, 13 bytes
    //         48, 11, // Var, 11 bytes
    //         6, 7, 43, 6, 1, 2, 1, 1, 6, // Oid 1.3.6.1.2.1.1.6
    //         129, 0, // NoSuchObject
    //     ];
    //     let msg = SnmpV1Message::try_from(data.as_ref())?;
    //     // community == public
    //     assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
    //     //
    //     match msg.pdu {
    //         SnmpPdu::GetResponse(pdu) => {
    //             assert_eq!(pdu.request_id, 0x5ebdd9ac);
    //             assert_eq!(pdu.vars.len(), 1);
    //             if let SnmpValue::NoSuchInstance = pdu.vars[0].value {
    //             } else {
    //                 return Err(SnmpError::InvalidData);
    //             }
    //         }
    //         _ => return Err(SnmpError::InvalidPdu),
    //     }
    //     //
    //     Ok(())
    // }
    // #[test]
    // fn test_parse_snmp_getresponse() -> SnmpResult<()> {
    //     let data = [
    //         48u8, 55, // Seq 55 bytes
    //         2, 1, 0, // INTEGER, v1
    //         4, 6, 112, 117, 98, 108, 105, 99, // "public"
    //         162, 42, // PDU, Get-Response, 42 bytes
    //         2, 4, 40, 86, 116, 146, // Request id
    //         2, 1, 0, // error status = 0
    //         2, 1, 0, // error index = 0
    //         48, 28, // Varbinds, 28 bytes
    //         48, 26, // Var item, 26 bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 6, 0, // OID 1.3.6.1.2.1.1.6.0
    //         4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115, 116, // String
    //     ];
    //     let msg = SnmpV1Message::try_from(data.as_ref())?;
    //     // community == public
    //     assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
    //     //
    //     match msg.pdu {
    //         SnmpPdu::GetResponse(pdu) => {
    //             assert_eq!(pdu.request_id, 0x28567492);
    //             assert_eq!(pdu.error_status, 0);
    //             assert_eq!(pdu.error_index, 0);
    //             assert_eq!(pdu.vars.len(), 1);
    //             assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
    //             match &pdu.vars[0].value {
    //                 SnmpValue::OctetString(s) => assert_eq!(
    //                     s.0,
    //                     &[71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115, 116]
    //                 ),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //         }
    //         _ => return Err(SnmpError::InvalidPdu),
    //     }
    //     Ok(())
    // }
    // #[test]
    // fn test_parse_snmp_getresponse_many() -> SnmpResult<()> {
    //     let data = [
    //         48u8, 129, 134, // Sequence, 134 bytes
    //         2, 1, 0, // ITEGER, v1
    //         4, 6, 112, 117, 98, 108, 105, 99, // OCTET STRING, "public"
    //         162, 121, // PDU, Get-Response, 121 byte
    //         2, 4, 91, 63, 155, 39, // Request ID, 0x5B3F9B27
    //         2, 1, 0, // error-status, 0
    //         2, 1, 0, // error-index, 0
    //         48, 107, // Varbinds, sequence, 107 bytes
    //         48, 22, // Var, sequence, 22 bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 2, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.2.0
    //         6, 10, 43, 6, 1, 4, 1, 191, 8, 3, 2,
    //         10, // OBJECT IDENTIFIER, 1.3.6.1.4.1.1.8072.3.2.10
    //         48, 16, // Var, sequence, 16  bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 3, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.3.0
    //         67, 4, 1, 53, 16, 171, // TimeTicks, 0x013510AB
    //         48, 26, // Var, sequennce, 26 bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 6, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.6.0
    //         4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115,
    //         116, // OCTET STRING
    //         48, 35, // Var, sequence, 35 bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 4, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.4.0
    //         4, 23, 116, 101, 115, 116, 32, 60, 116, 101, 115, 116, 64, 101, 120, 97, 109, 112, 108,
    //         101, 46, 99, 111, 109, 62, // OCTET STRING
    //     ];
    //     let msg = SnmpV1Message::try_from(data.as_ref())?;
    //     // community == public
    //     assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
    //     //
    //     match msg.pdu {
    //         SnmpPdu::GetResponse(pdu) => {
    //             assert_eq!(pdu.request_id, 0x5B3F9B27);
    //             assert_eq!(pdu.error_status, 0);
    //             assert_eq!(pdu.error_index, 0);
    //             assert_eq!(pdu.vars.len(), 4);
    //             // Var 0
    //             assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.2.0")?);
    //             match &pdu.vars[0].value {
    //                 SnmpValue::Oid(s) => {
    //                     assert_eq!(*s, SnmpOid::try_from("1.3.6.1.4.1.8072.3.2.10")?)
    //                 }
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 1
    //             assert_eq!(pdu.vars[1].oid, SnmpOid::try_from("1.3.6.1.2.1.1.3.0")?);
    //             match &pdu.vars[1].value {
    //                 SnmpValue::TimeTicks(s) => assert_eq!(s.0, 0x013510AB),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 2
    //             assert_eq!(pdu.vars[2].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
    //             match &pdu.vars[2].value {
    //                 SnmpValue::OctetString(s) => assert_eq!(s.0, b"Gufo SNMP Test"),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 3
    //             assert_eq!(pdu.vars[3].oid, SnmpOid::try_from("1.3.6.1.2.1.1.4.0")?);
    //             match &pdu.vars[3].value {
    //                 SnmpValue::OctetString(s) => assert_eq!(s.0, b"test <test@example.com>"),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //         }
    //         _ => return Err(SnmpError::InvalidPdu),
    //     }
    //     Ok(())
    // }
    // #[test]
    // fn test_parse_snmp_getresponse_many_rel() -> SnmpResult<()> {
    //     let data = [
    //         48u8, 116, // Sequence, 116 bytes
    //         2, 1, 0, // ITEGER, v1
    //         4, 6, 112, 117, 98, 108, 105, 99, // OCTET STRING, "public"
    //         162, 103, // PDU, Get-Response, 103 byte
    //         2, 4, 91, 63, 155, 39, // Request ID, 0x5B3F9B27
    //         2, 1, 0, // error-status, 0
    //         2, 1, 0, // error-index, 0
    //         48, 89, // Varbinds, sequence, 107 bytes
    //         48, 22, // Var, sequence, 22 bytes
    //         6, 8, 43, 6, 1, 2, 1, 1, 2, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.2.0
    //         6, 10, 43, 6, 1, 4, 1, 191, 8, 3, 2,
    //         10, // OBJECT IDENTIFIER, 1.3.6.1.4.1.1.8072.3.2.10
    //         48, 10, // Var, sequence, 10  bytes
    //         13, 2, 3, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[3.0]
    //         67, 4, 1, 53, 16, 171, // TimeTicks, 0x013510AB
    //         48, 20, // Var, sequennce, 20 bytes
    //         13, 2, 6, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[6.0]
    //         4, 14, 71, 117, 102, 111, 32, 83, 78, 77, 80, 32, 84, 101, 115,
    //         116, // OCTET STRING
    //         48, 29, // Var, sequence, 29 bytes
    //         13, 2, 4, 0, // OBJECT IDENTIFIER, 1.3.6.1.2.1.1.[4.0]
    //         4, 23, 116, 101, 115, 116, 32, 60, 116, 101, 115, 116, 64, 101, 120, 97, 109, 112, 108,
    //         101, 46, 99, 111, 109, 62, // OCTET STRING
    //     ];
    //     let msg = SnmpV1Message::try_from(data.as_ref())?;
    //     // community == public
    //     assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
    //     //
    //     match msg.pdu {
    //         SnmpPdu::GetResponse(pdu) => {
    //             assert_eq!(pdu.request_id, 0x5B3F9B27);
    //             assert_eq!(pdu.error_status, 0);
    //             assert_eq!(pdu.error_index, 0);
    //             assert_eq!(pdu.vars.len(), 4);
    //             // Var 0
    //             assert_eq!(pdu.vars[0].oid, SnmpOid::try_from("1.3.6.1.2.1.1.2.0")?);
    //             match &pdu.vars[0].value {
    //                 SnmpValue::Oid(s) => {
    //                     assert_eq!(*s, SnmpOid::try_from("1.3.6.1.4.1.8072.3.2.10")?)
    //                 }
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 1
    //             assert_eq!(pdu.vars[1].oid, SnmpOid::try_from("1.3.6.1.2.1.1.3.0")?);
    //             match &pdu.vars[1].value {
    //                 SnmpValue::TimeTicks(s) => assert_eq!(s.0, 0x013510AB),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 2
    //             assert_eq!(pdu.vars[2].oid, SnmpOid::try_from("1.3.6.1.2.1.1.6.0")?);
    //             match &pdu.vars[2].value {
    //                 SnmpValue::OctetString(s) => assert_eq!(s.0, b"Gufo SNMP Test"),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //             // Var 3
    //             assert_eq!(pdu.vars[3].oid, SnmpOid::try_from("1.3.6.1.2.1.1.4.0")?);
    //             match &pdu.vars[3].value {
    //                 SnmpValue::OctetString(s) => assert_eq!(s.0, b"test <test@example.com>"),
    //                 _ => return Err(SnmpError::InvalidData),
    //             }
    //         }
    //         _ => return Err(SnmpError::InvalidPdu),
    //     }
    //     Ok(())
    // }
    // #[test]
    // fn test_encode_snmp_get() -> SnmpResult<()> {
    //     let expected = [
    //         0x30u8, 0x35, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0,
    //         0x28, 0x02, 0x04, 0x63, 0xcc, 0xac, 0x7d, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30,
    //         0x1a, 0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x03, 0x05, 0x00,
    //         0x30, 0x0b, 0x06, 0x07, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x02, 0x05, 0x00,
    //     ];
    //     let community = [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63];
    //     let msg = SnmpV1Message {
    //         community: &community,
    //         pdu: SnmpPdu::GetRequest(SnmpGet {
    //             request_id: 0x63ccac7d,
    //             vars: vec![
    //                 SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 3]),
    //                 SnmpOid::from(vec![1, 3, 6, 1, 2, 1, 1, 2]),
    //             ],
    //         }),
    //     };
    //     let mut buf = Buffer::default();
    //     msg.push_ber(&mut buf)?;
    //     assert_eq!(buf.data(), &expected);
    //     Ok(())
    // }
}
