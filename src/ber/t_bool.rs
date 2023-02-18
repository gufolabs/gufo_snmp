// ------------------------------------------------------------------------
// Gufo SNMP: BER BOOLEAN class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, ToPython, TAG_BOOL};
use crate::error::SnmpError;
use pyo3::{IntoPy, Py, PyAny, Python};

pub struct SnmpBool(bool);

impl<'a> BerDecoder<'a> for SnmpBool {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_BOOL;

    // Implement X.690 pp 8.2: Encoding of a boolean value
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        if h.length != 1 {
            return Err(SnmpError::InvalidData);
        }
        Ok(SnmpBool(i[0] != 0))
    }
}

impl From<SnmpBool> for bool {
    fn from(value: SnmpBool) -> Self {
        value.0
    }
}

impl ToPython for &SnmpBool {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        Ok(self.0.into_py(py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_long() {
        let data = [1u8, 2, 0, 0];
        let r = SnmpBool::from_ber(&data);
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_ber() -> Result<(), Err<SnmpError>> {
        let data = [[1u8, 1, 0], [1, 1, 1], [1, 1, 255]];
        let expected = [false, true, true];
        for i in 0..data.len() {
            let (tail, v) = SnmpBool::from_ber(&data[i])?;
            assert_eq!(bool::from(v), expected[i]);
            assert_eq!(tail.len(), 0);
        }
        Ok(())
    }
}
