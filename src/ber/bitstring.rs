// ------------------------------------------------------------------------
// Gufo SNMP: BIT STRING type
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_BIT_STRING, Tag};
use crate::error::{SnmpError, SnmpResult};
use pyo3::{Bound, IntoPyObject, PyAny, Python};

pub struct SnmpBitString(u64);

impl<'a> BerDecoder<'a> for SnmpBitString {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_BIT_STRING;

    // Implement X.690 pp 8.6: Encoding of a bitstring value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        if h.length == 0 || h.length > 10 {
            return Err(SnmpError::InvalidData);
        }
        if h.length == 1 {
            return Ok(Self(0));
        }
        // data is: <unused bits> <payload>
        let unused = i[0];
        let payload = &i[1..];
        // Align to 8 bytes
        let mut buf = [0u8; 8];
        let start = 8 - payload.len();
        buf[start..].copy_from_slice(payload);
        // Decode value
        let value = u64::from_be_bytes(buf);
        Ok(Self(value >> unused))
    }
}

impl<'py> IntoPyObject<'py> for &SnmpBitString {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_pyobject(py)?.into_any())
    }
}

impl From<SnmpBitString> for u64 {
    fn from(value: SnmpBitString) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [
            vec![3, 1, 0],                                        // 0
            vec![3, 7, 0x04, 0x0A, 0x3B, 0x5F, 0x29, 0x1C, 0xD0], // 0x0A3B5F291CD
            vec![3, 3, 0x04, 0xB0, 0x90],                         // 0xB09
            vec![3, 2, 0x5, 0xA0],                                //5
        ];
        let expected = [0, 0x0A3B5F291CD, 0xB09, 5];
        for i in 0..data.len() {
            let (tail, v) = SnmpBitString::from_ber(&data[i])?;
            assert_eq!(tail.len(), 0);
            assert_eq!(v.0, expected[i]);
        }
        Ok(())
    }
}
