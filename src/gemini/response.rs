use super::Header;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[derive(Clone)]
#[cfg_attr(all(feature = "py_bindings"), pyclass())]
pub struct Response {
    pub header: Header,
    pub body: Option<Vec<u8>>
}

impl Response {
    pub fn new(header: Header, body: Option<Vec<u8>>) -> Response {
        Response {
            header,
            body
        }
    }
}