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

// cfg_attr doesn't work with getters and setters
#[cfg(feature = "py_bindings")]
#[pymethods]
impl Response {
    #[getter(header)]
    pub fn py_header(&self) -> PyResult<Header> {
        Ok(self.header.clone())
    }

    #[getter(body)]
    pub fn py_body(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.body.clone())
    }
}