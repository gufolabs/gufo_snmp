// ------------------------------------------------------------------------
// Gufo SNMP: BER BOOLEAN class
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_BOOL, Tag};
use crate::error::{SnmpError, SnmpResult};
use pyo3::{Bound, IntoPyObject, PyAny, Python, types::PyBool};

pub struct SnmpBool(bool);

impl<'a> BerDecoder<'a> for SnmpBool {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_BOOL;

    // Implement X.690 pp 8.2: Encoding of a boolean value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
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

impl<'py> IntoPyObject<'py> for &SnmpBool {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBool::new(py, self.0).to_owned().into_any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long() {
        let data = [1u8, 2, 0, 0];
        let r = SnmpBool::from_ber(&data);
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
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
