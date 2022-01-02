use std::io;
use thiserror::Error;
use url::ParseError;
use native_tls::Error as TLSError;
use native_tls::HandshakeError;

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
    #[error("TCP connection error: {0}")]
    TCPConnect(io::Error),
    #[error("TLS handshake error: {0}")]
    TLSHandshake(Box<HandshakeError<std::net::TcpStream>>),
    #[error("Failed to create TLS connector: {0}")]
    TLSConnector(TLSError),
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
            Error::TCPConnect(_) | Error::TLSHandshake(_) | Error::TLSConnector(_) |
            Error::StreamIO(_, _) => {
                PyIOError::new_err(err.to_string())
            }
        }
    }
}