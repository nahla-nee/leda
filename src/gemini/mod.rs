//! A module with all the gemini protocol functionality.

/// Make gemini requests and parse their responses.
mod client;
/// Represent a gemini response.
mod response;
/// Represent a gemini response's header.
pub mod header;
/// Represent and parse gemtext documents.
pub mod gemtext;
/// The error type returned by functions in this module.
mod error;

pub use client::{Client, ClientBuilder};
pub use response::Response;
#[cfg(feature = "py_bindings")]
pub use gemtext::{PyGemtext, PyGemtextElement};
pub use error::Error;