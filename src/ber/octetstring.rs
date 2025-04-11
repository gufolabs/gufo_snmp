// ------------------------------------------------------------------------
// Gufo SNMP: OCTET STRING type
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_OCTET_STRING, Tag};
use crate::error::{SnmpError, SnmpResult};
use pyo3::{Bound, IntoPyObject, PyAny, Python, types::PyBytes};

pub struct SnmpOctetString<'a>(pub(crate) &'a [u8]);

impl<'a> BerDecoder<'a> for SnmpOctetString<'a> {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_OCTET_STRING;

    // Implement X.690 pp 8.7: Encoding of an octetstring value
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        Ok(SnmpOctetString(&i[..h.length]))
    }
}

impl<'a, 'py> IntoPyObject<'py> for &'a SnmpOctetString<'a> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, self.0).into_any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ber() -> SnmpResult<()> {
        let data = [4u8, 5, 0, 1, 2, 3, 4];
        let (tail, s) = SnmpOctetString::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(s.0, &data[2..]);
        Ok(())
    }
}
