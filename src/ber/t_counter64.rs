// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Application Class Counter64
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, ToPython, TAG_APP_COUNTER64};
use crate::error::SnmpResult;
use pyo3::{IntoPy, Py, PyAny, Python};

pub struct SnmpCounter64(pub(crate) u64);

impl<'a> BerDecoder<'a> for SnmpCounter64 {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_APP_COUNTER64;

    // Implement RFC
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        let v = i
            .iter()
            .take(h.length)
            .map(|x| *x as u64)
            .reduce(|acc, x| (acc << 8) | x)
            .unwrap_or(0);
        Ok(SnmpCounter64(v))
    }
}

impl ToPython for &SnmpCounter64 {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>> {
        Ok(self.0.into_py(py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse1() -> SnmpResult<()> {
        let data = [0x46, 0x4, 0, 0x89, 0x92, 0xDB];
        let (tail, tt) = SnmpCounter64::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(tt.0, 0x008992DB);
        Ok(())
    }
    #[test]
    fn test_parse2() -> SnmpResult<()> {
        let data = [0x46, 4, 1, 53, 16, 171];
        let (tail, tt) = SnmpCounter64::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(tt.0, 0x013510AB);
        Ok(())
    }
}
