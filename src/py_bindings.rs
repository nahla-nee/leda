use std::time::Duration;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pyo3::exceptions::{PyIOError, PyValueError};

use super::gemini::{
    gemtext::Gemtext,
    header::{CertFailCode, FailPermanentCode, FailTemporaryCode, InputCode,
        RedirectCode, StatusCode},
    Client,
    Error,
    Response
};

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
        } else {
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
            StatusCode::CertFail(CertFailCode::CertNotValid) => 62,
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
pub struct PyGemtext {}

#[pymethods]
impl PyGemtext {
    #[staticmethod]
    pub fn to_html(input: &str) -> Result<String, Error> {
        let gemtext = Gemtext::new(input)?;
        Ok(gemtext.to_html())
    }
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> Self {
        match err {
            Error::HeaderFormat(_)
            | Error::UrlParse(_)
            | Error::UrlNoHost(_)
            | Error::GemtextFormat(_)
            | Error::UrlNoAddress(_) => PyValueError::new_err(err.to_string()),
            Error::TCPConnect(_) | Error::TLSClient(_) | Error::StreamIO(_, _) => {
                PyIOError::new_err(err.to_string())
            }
        }
    }
}

#[pymodule]
pub(crate) fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<Response>()?;
    m.add_class::<PyGemtext>()?;

    Ok(())
}

#[pymodule]
pub(crate) fn leda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(gemini))?;

    Ok(())
}
