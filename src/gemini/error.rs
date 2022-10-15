use std::io;
use thiserror::Error;
use url::ParseError;

/// Represents the different error types this submodule returns
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
    TLSClient(rustls::Error),
    #[error("Couldn't connect to address {1}, TCP connection error: {0}")]
    TCPConnect(io::Error, String),
    #[error("Stream IO failure, {0}: {1}")]
    StreamIO(&'static str, io::Error),
    #[error("Malformed gemtext document: {0}")]
    GemtextFormat(String),
}
