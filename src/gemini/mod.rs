//! A module with all the gemini protocol functionality.

/// Make gemini requests and parse their responses.
pub mod client;
/// Represent a gemini response.
pub mod response;
/// Represent a gemini response's header.
pub mod header;
/// Represent and parse gemtext documents.
pub mod gemtext;
/// The error type returned by functions in this module.
pub mod error;

pub use client::Client;
pub use header::Header;
pub use response::Response;
#[cfg(feature = "py_bindings")]
pub use gemtext::{PyGemtext, PyGemtextElement};
pub use error::Error;