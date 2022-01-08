use super::Header;
// make the docs code happy
#[allow(unused_imports)]
use super::header;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;


/// Represents a response generated from a gemini server.
#[derive(Clone)]
#[cfg_attr(all(feature = "py_bindings"), pyclass)]
pub struct Response {
    /// The header the server responded with, includes the response status code as well as the meta
    /// information provided.
    pub header: Header,
    /// The response body content from the server. `body` will only be `Some` if the header's
    /// [`Header::status`] is [`header::StatusCode::Success`], otherwise it'll be `None`.
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
    pub fn py_header(&self) -> PyResult<(u32, String)> {
        let status = self.header.status.to_u32();
        let meta = self.header.meta.clone();

        Ok((status, meta))
    }

    #[getter(body)]
    pub fn py_body(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.body.clone())
    }
}