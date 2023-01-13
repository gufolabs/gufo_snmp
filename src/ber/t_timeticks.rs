// ------------------------------------------------------------------------
// Gufo Snmp: SNMP Application Class TimeTicks
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, ToPython, TAG_APP_TIMETICKS};
use crate::error::SnmpError;
use pyo3::{types::PyLong, IntoPy, Py, PyAny, Python};

pub(crate) struct SnmpTimeTicks(pub(crate) u32);

impl<'a> BerDecoder<'a> for SnmpTimeTicks {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_APP_TIMETICKS;

    // Implement RFC
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        let mut v = 0u32;
        for n in i[..h.length].iter() {
            v = (v << 8) | (*n as u32);
        }
        Ok(SnmpTimeTicks(v))
    }
}

impl ToPython for &SnmpTimeTicks {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        Ok(self.0.into_py(py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse() -> Result<(), Err<SnmpError>> {
        let data = [0x43, 0x4, 0, 0x89, 0x92, 0xDB];
        let (tail, tt) = SnmpTimeTicks::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(tt.0, 0x008992DB);
        Ok(())
    }
}
