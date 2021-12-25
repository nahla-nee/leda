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
    #[error("TCP connection error: {0}")]
    TCPConnect(io::Error),
    #[error("TLS handshake error: {0}")]
    TLSHandshake(Box<HandshakeError<std::net::TcpStream>>),
    #[error("Failed to create TLS connector: {0}")]
    TLSConnector(TLSError),
    #[error("Stream IO failure, {0}: {1}")]
    StreamIO(&'static str, io::Error)
}