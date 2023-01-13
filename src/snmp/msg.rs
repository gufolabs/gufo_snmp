// ------------------------------------------------------------------------
// Gufo Snmp: SNMP Message
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

pub(crate) struct SnmpMessage<'a> {
    pub(crate) version: SnmpVersion,
    pub(crate) community: &'a [u8],
    pub(crate) pdu: SnmpPdu<'a>,
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
        buf.push_u8(TAG_OCTET_STRING as u8)?;
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
    use crate::snmp::var::SnmpValue;

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
    fn test_parse_snmpv2_getresponse_stripped() -> Result<(), SnmpError> {
        let data = [
            48u8, 40, // Seq 40 bytes
            2, 1, 1, // INTEGER 1, v2C
            4, 6, 112, 117, 98, 108, 105, 99, // "public"
            162, 27, // 27 bytes
            2, 4, 94, 189, 217, 172, // Request id
            2, 1, 0, // Error status = 0
            2, 1, 0, // Error index = 0
            48, 13, // Varbinds, 13 bytes
            48, 11, // Var, 11 bytes
            6, 7, 43, 6, 1, 2, 1, 1, 6, // Oid 1.3.6.1.2.1.1.6
            129, 0, //
        ];
        let msg = SnmpMessage::try_from(data.as_ref())?;
        // Check version
        assert_eq!(msg.version, SnmpVersion::V2C);
        // community == public
        assert_eq!(msg.community, [0x70u8, 0x75, 0x62, 0x6c, 0x69, 0x63]);
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
