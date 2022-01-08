use std::io;
use thiserror::Error;
use url::ParseError;

/// Represents the different error types this submodule returns
#[derive(Error, Debug)]
pub enum Error {
    /// Returned if there was a problem parsing the header sent back from a gemini server's response.
    #[error("Header is malformed: {0}")]
    HeaderFormat(String),
    /// Returned if there was a problem parsing the URL passed to a function
    #[error("Failed to parse URL: {0}")]
    UrlParse(ParseError),
    /// Returned if the URL passed to a function didn't contain a host.
    #[error("The given URL didn't have a host: {0}")]
    UrlNoHost(String),
    /// Returned if the URL passsed to a function couldn't be resolved to an address to connect to.
    #[error("The URL couldn't be resolved to an address: {0}")]
    UrlNoAddress(String),
    /// Returned if there was a problem with creating a TLS client with which to make connections
    #[error("Failed to create TLS client: {0}")]
    TLSClient(openssl::error::ErrorStack),
    /// Returned if there was a problem establishing a TCP connection.
    #[error("TCP connection error: {0}")]
    TCPConnect(io::Error),
    /// Returned in case of general IO failures.
    #[error("Stream IO failure, {0}: {1}")]
    StreamIO(&'static str, io::Error),
    /// Returned if there was a problem with parsing gemtext.
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
            Error::TCPConnect(_) | Error::TLSClient(_) | Error::StreamIO(_, _) => {
                PyIOError::new_err(err.to_string())
            }
        }
    }
}