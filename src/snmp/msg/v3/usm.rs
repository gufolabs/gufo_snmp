// ------------------------------------------------------------------------
// Gufo SNMP: SNMP v3 User-based Security Model (USM)
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{
    BerDecoder, BerEncoder, SnmpInt, SnmpOctetString, SnmpSequence, TAG_OCTET_STRING,
};
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};

pub struct UsmParameters<'a> {
    pub engine_id: &'a [u8],
    pub engine_boots: i64,
    pub engine_time: i64,
    pub user_name: &'a [u8],
    pub auth_params: &'a [u8],
    pub privacy_params: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for UsmParameters<'a> {
    type Error = SnmpError;

    fn try_from(i: &'a [u8]) -> SnmpResult<UsmParameters<'a>> {
        // Top-level sequence
        let (tail, envelope) = SnmpSequence::from_ber(i)?;
        if !tail.is_empty() {
            return Err(SnmpError::TrailingData);
        }
        // Engine id
        let (tail, engine_id) = SnmpOctetString::from_ber(envelope.0)?;
        // Engine boots
        let (tail, engine_boots) = SnmpInt::from_ber(tail)?;
        // Engine time
        let (tail, engine_time) = SnmpInt::from_ber(tail)?;
        // User name
        let (tail, user_name) = SnmpOctetString::from_ber(tail)?;
        // Auth parameters
        let (tail, auth_parameters) = SnmpOctetString::from_ber(tail)?;
        // Privacy parameters
        let (_, privacy_parameters) = SnmpOctetString::from_ber(tail)?;
        Ok(UsmParameters {
            engine_id: engine_id.0,
            engine_boots: engine_boots.into(),
            engine_time: engine_time.into(),
            user_name: user_name.0,
            auth_params: auth_parameters.0,
            privacy_params: privacy_parameters.0,
        })
    }
}

const EMPTY_BER: [u8; 2] = [TAG_OCTET_STRING, 0];

impl<'a> BerEncoder for UsmParameters<'a> {
    fn push_ber(&self, buf: &mut Buffer) -> SnmpResult<()> {
        let l0 = buf.len();
        // Push privacy parameters
        if self.privacy_params.is_empty() {
            buf.push(&EMPTY_BER)?;
        } else {
            buf.push_tagged(TAG_OCTET_STRING, self.privacy_params)?;
        }
        // Push auth parameters
        if self.auth_params.is_empty() {
            buf.push(&EMPTY_BER)?;
        } else {
            buf.push_tagged(TAG_OCTET_STRING, self.auth_params)?;
            // Place bookmark for further signing
            buf.set_bookmark(2);
        }
        // Push user name
        buf.push_tagged(TAG_OCTET_STRING, self.user_name)?;
        // Push engine time
        let engine_time: SnmpInt = self.engine_time.into();
        engine_time.push_ber(buf)?;
        // Push engine boots
        let engine_boots: SnmpInt = self.engine_boots.into();
        engine_boots.push_ber(buf)?;
        // Push engine id
        if self.engine_id.is_empty() {
            buf.push(&EMPTY_BER)?;
        } else {
            buf.push_tagged(TAG_OCTET_STRING, self.engine_id)?;
        }
        // Push top-level sequence
        buf.push_tag_len(0x30, buf.len() - l0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_usm_plain() -> SnmpResult<()> {
        let data = [
            0x30, 0x13, 0x04, 0x00, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x04, 0x05, 0x61, 0x64,
            0x6d, 0x69, 0x6e, 0x04, 0x00, 0x04, 0x00,
        ];
        let empty: [u8; 0] = [];
        let usm = UsmParameters::try_from(data.as_ref())?;
        assert_eq!(usm.engine_id, empty);
        // assert_eq!(usm.engine_boots, 0);
        // assert_eq!(usm.engine_time, 0);
        assert_eq!(usm.user_name, "admin".as_bytes());
        assert_eq!(usm.auth_params, empty);
        assert_eq!(usm.privacy_params, empty);
        Ok(())
    }
    #[test]
    fn test_push_usm_plain() -> SnmpResult<()> {
        let expected = [
            0x30, 0x13, 0x04, 0x00, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x04, 0x05, 0x61, 0x64,
            0x6d, 0x69, 0x6e, 0x04, 0x00, 0x04, 0x00,
        ];
        let empty: [u8; 0] = [];
        let usm = UsmParameters {
            engine_id: &empty,
            engine_boots: 0,
            engine_time: 0,
            user_name: "admin".as_bytes(),
            auth_params: &empty,
            privacy_params: &empty,
        };
        let mut buf = Buffer::default();
        usm.push_ber(&mut buf)?;
        assert_eq!(buf.data(), &expected);
        Ok(())
    }
}
