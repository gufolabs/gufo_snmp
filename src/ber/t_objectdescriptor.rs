// ------------------------------------------------------------------------
// Gufo SNMP: OBJECT DESCRIPTOR type
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, Tag, ToPython, TAG_OBJECT_DESCRIPTOR};
use crate::error::SnmpResult;
use pyo3::types::PyBytes;
use pyo3::{Py, PyAny, Python};

pub struct SnmpObjectDescriptor<'a>(pub(crate) &'a [u8]);

impl<'a> BerDecoder<'a> for SnmpObjectDescriptor<'a> {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = true;
    const TAG: Tag = TAG_OBJECT_DESCRIPTOR;

    // Implement X.690 pp 8.7: Encoding of an octetstring value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        Ok(SnmpObjectDescriptor(&i[..h.length]))
    }
}

impl ToPython for &SnmpObjectDescriptor<'_> {
    fn try_to_python(self, py: Python) -> SnmpResult<Py<PyAny>> {
        let v = PyBytes::new(py, self.0);
        Ok(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [7u8, 5, 0, 1, 2, 3, 4];
        let (tail, s) = SnmpObjectDescriptor::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(s.0, &data[2..]);
        Ok(())
    }
}
