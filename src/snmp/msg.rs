// ------------------------------------------------------------------------
// Gufo Snmp: SNMP Message
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------
use super::pdu::SnmpPdu;
use super::SnmpVersion;
use crate::ber::{BerDecoder, SnmpInt, SnmpOctetString, SnmpSequence};
use crate::error::SnmpError;

pub(crate) struct SnmpMessage<'a> {
    version: SnmpVersion,
    community: &'a [u8],
    pdu: SnmpPdu<'a>,
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
        let version = match v_code.as_i64() {
            0 => SnmpVersion::V1,
            1 => SnmpVersion::V2C,
            _ => return Err(SnmpError::UnknownVersion),
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ber::SnmpOid;

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
            SnmpPdu::Get(pdu) => {
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
            SnmpPdu::Get(pdu) => {
                assert_eq!(pdu.request_id, 0x63ccac7d);
                assert_eq!(pdu.vars, vars);
            }
            _ => return Err(SnmpError::InvalidPdu),
        }
        Ok(())
    }
}
