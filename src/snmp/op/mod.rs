// ------------------------------------------------------------------------
// Gufo SNMP: Python interface for requests and reply
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

pub mod get;
pub mod getbulk;
pub mod getiter;
pub mod getmany;
pub mod getnext;

use super::msg::SnmpPdu;
pub use get::OpGet;
pub use getbulk::OpGetBulk;
pub use getiter::GetIter;
pub use getmany::OpGetMany;
pub use getnext::OpGetNext;
use pyo3::prelude::*;

pub trait PyOp<'a, T>
where
    T: 'a,
{
    fn from_python(obj: T, request_id: i64) -> PyResult<SnmpPdu<'a>>;
    fn to_python(pdu: &SnmpPdu, iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject>;
}
