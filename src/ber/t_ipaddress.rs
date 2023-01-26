// ------------------------------------------------------------------------
// Gufo Snmp: SNMP Application Class IpAddress
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{BerDecoder, BerHeader, ToPython, TAG_APP_IPADDRESS};
use crate::error::SnmpError;
use pyo3::types::PyString;
use pyo3::{Py, PyAny, Python};

pub(crate) struct SnmpIpAddress(u8, u8, u8, u8);

impl<'a> BerDecoder<'a> for SnmpIpAddress {
    const IS_CONSTRUCTED: bool = false;
    const TAG: usize = TAG_APP_IPADDRESS;

    // Implement RFC
    fn decode(i: &'a [u8], h: &BerHeader) -> Result<Self, SnmpError> {
        if h.length != 4 {
            return Err(SnmpError::InvalidTagFormat);
        }
        Ok(SnmpIpAddress(i[0], i[1], i[2], i[3]))
    }
}

impl Into<String> for &SnmpIpAddress {
    fn into(self) -> String {
        format!("{}.{}.{}.{}", self.0, self.1, self.2, self.3)
    }
}

impl ToPython for &SnmpIpAddress {
    fn try_to_python(self, py: Python) -> Result<Py<PyAny>, SnmpError> {
        Ok(PyString::new(py, &<&SnmpIpAddress as Into<String>>::into(self)).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;

    #[test]
    fn test_parse() -> Result<(), Err<SnmpError>> {
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
        let ip = SnmpIpAddress(127, 0, 0, 1);
        assert_eq!(<&SnmpIpAddress as Into<String>>::into(&ip), "127.0.0.1");
    }
}
