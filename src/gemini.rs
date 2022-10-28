//! A module with all the gemini protocol functionality.

/// Make gemini requests and parse their responses.
mod client;
/// The error type returned by functions in this module.
mod error;
/// Represent and parse gemtext documents.
pub mod gemtext;
/// Represent a gemini response's header.
pub mod header;
/// Represent a gemini response.
mod response;

pub use client::Client;
pub use error::Error;
pub use response::Response;
