pub mod client;
pub mod header;
pub mod response;
pub mod gemtext;
pub mod error;

pub use client::Client;
pub use header::Header;
pub use response::Response;
pub use gemtext::{Gemtext, GemtextElement};
#[cfg(feature = "py_bindings")]
pub use gemtext::{PyGemtext, PyGemtextElement};
pub use error::Error;