use std::io;
use thiserror::Error;
use url::ParseError;
use webpki;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Header is malformed: {0}")]
    HeaderFormat(String),
    #[error("Failed to parse URL: {0}")]
    UrlParse(ParseError),
    #[error("The given URL didn't have a host: {0}")]
    UrlNoHost(String),
    #[error("The URL couldn't be resolved to an address: {0}")]
    UrlNoAddress(String),
    #[error("Failed to create TLS client: {0}")]
    TLSClient(io::Error),
    #[error("Failed to add certificate to client config: {0}")]
    TLSCert(webpki::Error),
    #[error("TCP connection error: {0}")]
    TCPConnect(io::Error),
    #[error("Stream IO failure, {0}: {1}")]
    StreamIO(&'static str, io::Error),
    #[error("Malformed gemtext document: {0}")]
    GemtextFormat(String)
}

#[cfg(feature = "py_bindings")]
use pyo3::{prelude::*, exceptions::{PyIOError, PyValueError}};

#[cfg(feature = "py_bindings")]
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> Self {
        match err {
            Error::HeaderFormat(_) | Error::UrlParse(_) | Error::UrlNoHost(_) |
            Error::GemtextFormat(_) | Error::UrlNoAddress(_) => {
                PyValueError::new_err(err.to_string())
            },
            Error::TCPConnect(_) | Error::TLSClient(_) | Error::StreamIO(_, _) |
            Error::TLSCert(_) => {
                PyIOError::new_err(err.to_string())
            }
        }
    }
}