use std::time::Duration;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

use super::gemini::{*, gemtext::*, header::*};

#[pymethods]
impl Client {
    #[new]
    pub fn __new__() -> Result<Client, Error> {
        Client::new()
    }

    #[pyo3(name = "set_timeout")]
    pub fn py_set_timeout(&mut self, seconds: u64) {
        if seconds == 0 {
            self.set_timeout(None);
        }
        else {
            self.set_timeout(Some(Duration::from_secs(seconds)));
        }
    }

    #[pyo3(name = "request")]
    pub fn py_request(&mut self, url: String) -> Result<Response, Error> {
        self.request(url)
    }
}

#[pymethods]
impl Response {
    #[getter(header)]
    pub fn py_header(&self) -> PyResult<(u32, String)> {
        let status = match self.header.status {
            StatusCode::Input(InputCode::Input) => 10,
            StatusCode::Input(InputCode::Sensitive) => 11,
            StatusCode::Success => 20,
            StatusCode::Redirect(RedirectCode::Temporary) => 30,
            StatusCode::Redirect(RedirectCode::Permanent) => 31,
            StatusCode::FailTemporary(FailTemporaryCode::Temporary) => 40,
            StatusCode::FailTemporary(FailTemporaryCode::ServerUnavailable) => 41,
            StatusCode::FailTemporary(FailTemporaryCode::CGIError) => 42,
            StatusCode::FailTemporary(FailTemporaryCode::ProxyError) => 43,
            StatusCode::FailTemporary(FailTemporaryCode::SlowDown) => 44,
            StatusCode::FailPermanent(FailPermanentCode::Permanent) => 50,
            StatusCode::FailPermanent(FailPermanentCode::NotFound) => 51,
            StatusCode::FailPermanent(FailPermanentCode::Gone) => 52,
            StatusCode::FailPermanent(FailPermanentCode::ProxyRefused) => 53,
            StatusCode::FailPermanent(FailPermanentCode::BadRequest) => 59,
            StatusCode::CertFail(CertFailCode::CertRequired) => 60,
            StatusCode::CertFail(CertFailCode::CertNotAuthorized) => 61,
            StatusCode::CertFail(CertFailCode::CertNotValid) => 62
        };
        let meta = self.header.meta.clone();

        Ok((status, meta))
    }

    #[getter(body)]
    pub fn py_body(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.body.clone())
    }
}


#[pyclass(name = "Gemtext")]
#[derive(Clone)]
pub struct PyGemtext {
    #[pyo3(get)]
    elements: Vec<PyGemtextElement>
}


#[pyclass(name = "GemtextElement")]
#[derive(Clone)]
pub struct PyGemtextElement {
    #[pyo3(get)]
    format: u32, // pyo3 doesn't have great enum representation so python just has to deal with u32
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    link_text: Option<String> // only Some if format is a Link
}

#[pymethods]
impl PyGemtext {
    #[new]
    pub fn new(input: &str) -> Result<PyGemtext, Error> {
        let gemtext = Gemtext::new(input)?;
        let mut elements = Vec::with_capacity(gemtext.elements.len());

        for element in gemtext.elements {
            let values = match element {
                Element::Text(text) => (0, text.to_string(), None),
                Element::Link(text, link) => (1, text.to_string(), Some(link.to_string())),
                Element::Heading(text) => (2, text.to_string(), None),
                Element::Subheading(text) => (3, text.to_string(), None),
                Element::Subsubheading(text) => (4, text.to_string(), None),
                Element::UnorederedListItem(text) => (5, text.to_string(), None),
                Element::BlockQuote(text) => (6, text.to_string(), None),
                Element::Preformatted(text) => (7, text.to_string(), None)
            };

            elements.push(PyGemtextElement::new(values.0, values.1, values.2));
        };

        Ok(PyGemtext{
            elements
        })
    }

    #[staticmethod]
    pub fn parse_to_html(input: &str) -> Result<String, Error> {
        Gemtext::parse_to_html(input)
    }
}

#[cfg(feature = "py_bindings")]
impl PyGemtextElement {
    pub fn new(format: u32, value: String, link_text: Option<String>) -> PyGemtextElement {
        PyGemtextElement {
            format,
            value,
            link_text
        }
    }
}

#[pymodule]
pub(crate) fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<Response>()?;
    m.add_class::<PyGemtext>()?;
    m.add_class::<PyGemtextElement>()?;

    Ok(())
}

#[pymodule]
pub(crate) fn leda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(gemini))?;

    Ok(())
}