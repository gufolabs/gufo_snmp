// ------------------------------------------------------------------------
// Gufo SNMP: Python interface for requests and reply
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

pub mod get;
pub mod getbulk;
pub mod getiter;
pub mod getmany;
pub mod getnext;
pub mod refresh;

use super::msg::SnmpPdu;
pub use get::OpGet;
pub use getbulk::OpGetBulk;
pub use getiter::GetIter;
pub use getmany::OpGetMany;
pub use getnext::OpGetNext;
use pyo3::prelude::*;
pub use refresh::OpRefresh;

pub trait PyOp<'a, T>
where
    T: 'a,
{
    fn from_python(obj: T, request_id: i64) -> PyResult<SnmpPdu<'a>>;
    fn to_python<'py>(
        pdu: &SnmpPdu,
        iter: Option<&mut GetIter>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>>;
}
