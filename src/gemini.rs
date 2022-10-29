//! A module with all the gemini protocol functionality.

/// Make gemini requests and parse their responses.
mod client;
/// Represent and parse gemtext documents.
pub mod gemtext;
/// Represent a gemini response's header.
pub mod header;
/// Represent a gemini response.
mod response;

pub use client::Client;
pub use header::Header;
pub use gemtext::Gemtext;
pub use response::Response;

use thiserror::Error;

/// Represents the different error types this module returns
#[derive(Error, Debug)]
pub enum Error {
    #[error("Header is malformed: {0}")]
    HeaderFormat(String),
    #[error("Failed to parse URL: {0}")]
    UrlParse(url::ParseError),
    #[error("The given URL didn't have a host: {0}")]
    UrlNoHost(String),
    #[error("The URL couldn't be resolved to an address: {0}")]
    UrlNoAddress(String),
    #[error("Failed to create TLS client: {0}")]
    TLSClient(rustls::Error),
    #[error("Couldn't connect to address {1}, TCP connection error: {0}")]
    TCPConnect(std::io::Error, String),
    #[error("Stream IO failure, {0}: {1}")]
    StreamIO(&'static str, std::io::Error),
    #[error("Malformed gemtext document: {0}")]
    GemtextFormat(String),
}
