// ------------------------------------------------------------------------
// Gufo SNMP: SNMP Application Class IpAddress
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, TAG_APP_IPADDRESS, Tag};
use crate::error::{SnmpError, SnmpResult};
use pyo3::{Bound, IntoPyObject, PyAny, Python, types::PyString};

pub struct SnmpIpAddress(u8, u8, u8, u8);

impl<'a> BerDecoder<'a> for SnmpIpAddress {
    const ALLOW_PRIMITIVE: bool = true;
    const ALLOW_CONSTRUCTED: bool = false;
    const TAG: Tag = TAG_APP_IPADDRESS;

    // Implement RFC
    fn decode(i: &'a [u8], h: &BerHeader) -> SnmpResult<Self> {
        if h.length != 4 {
            return Err(SnmpError::InvalidTagFormat);
        }
        Ok(SnmpIpAddress(i[0], i[1], i[2], i[3]))
    }
}

impl From<&SnmpIpAddress> for String {
    fn from(value: &SnmpIpAddress) -> Self {
        format!("{}.{}.{}.{}", value.0, value.1, value.2, value.3)
    }
}

impl<'py> IntoPyObject<'py> for &SnmpIpAddress {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = SnmpError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s: String = self.into();
        Ok(PyString::new(py, &s).into_any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> SnmpResult<()> {
        let data = [0x40, 0x4, 127, 0, 0, 1];
        let (tail, ip) = SnmpIpAddress::from_ber(&data)?;
        assert_eq!(tail.len(), 0);
        assert_eq!(ip.0, 127);
        assert_eq!(ip.1, 0);
        assert_eq!(ip.2, 0);
        assert_eq!(ip.3, 1);
        Ok(())
    }

    #[test]
    fn test_into_str() {
        let ip = &SnmpIpAddress(127, 0, 0, 1);
        let s: String = ip.into();
        assert_eq!(s, "127.0.0.1");
    }
}
