// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Application Class Opaque
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, ToPython, TAG_APP_OPAQUE};
use crate::error::SnmpResult;
use pyo3::types::PyBytes;
use pyo3::{Py, PyAny, Python};

pub struct SnmpOpaque<'a>(pub(crate) &'a [u8]);

impl<'a> BerDecoder<'a> for SnmpOpaque<'a> {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_APP_OPAQUE;

    // Implement X.690 pp 8.7: Encoding of an Opaque value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        Ok(SnmpOpaque(&i[..h.length]))
    }
}

impl<'a> ToPython for &SnmpOpaque<'a> {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>> {
        let v = PyBytes::new_bound(py, self.0);
        Ok(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [0x44, 5, 0, 1, 2, 3, 4];
        let (tail, s) = SnmpOpaque::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(s.0, &data[2..]);
        Ok(())
    }
}
