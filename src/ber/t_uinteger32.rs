// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Application Class UInteger32
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, ToPython, TAG_APP_UINTEGER32};
use crate::error::SnmpError;
use pyo3::{IntoPy, Py, PyAny, Python};

pub struct SnmpUInteger32(pub(crate) u32);

impl<'a> BerDecoder<'a> for SnmpUInteger32 {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_APP_UINTEGER32;

    // Implement RFC
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        let mut v = 0u32;
        for &n in i[..h.length].iter() {
            v = (v << 8) | (n as u32);
        }
        Ok(SnmpUInteger32(v))
    }
}

impl ToPython for &SnmpUInteger32 {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        Ok(self.0.into_py(py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse1() -> Result<(), Err<SnmpError>> {
        let data = [0x47, 0x4, 0, 0x89, 0x92, 0xDB];
        let (tail, tt) = SnmpUInteger32::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(tt.0, 0x008992DB);
        Ok(())
    }
    #[test]
    fn test_parse2() -> Result<(), Err<SnmpError>> {
        let data = [0x47, 4, 1, 53, 16, 171];
        let (tail, tt) = SnmpUInteger32::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(tt.0, 0x013510AB);
        Ok(())
    }
}
