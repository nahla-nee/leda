//! A crate that implements the client logic for several small internet protocols. Currently only
//! supports gemini but with plans to support other protocols such as finger, and gopher.
//! 
//! ## Cargo Features
//! 
//! - **py_bindings**: generates bindings for python
//! 
//! ## Get started
//! 
//! This is a minimal example to show what using this library is like.
//! 
//! ```no_run
//! use leda::gemini;
//! 
//! fn main() {
//!     let url = String::from("gemini://gemini.circumlunar.space/")
//! 
//!     let mut client = gemini::Client::with_timeout(Duration::from_secs(5))
//!         .expect("Failed to create gemini client");
//! 
//!     let response = client.request(url)
//!         .expect("Failed to retrieve gemini page");
//! 
//!     let body = &response.body.expect("Body was none!");
//!     let body = std::str::from_utf8(body)
//!         .expect("Failed to parse body as utf8");
//!     let html = gemini::gemtext::Gemtext::parse_to_html(body)
//!         .expect("Failed to parse body as gemtext");
//! 
//!     println!("raw body: \n{}\n", body);
//!     println!("html body: \n{}\n", html);
//! }
//! ```

pub mod gemini;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;


#[cfg(feature = "py_bindings")]
#[pymodule]
fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gemini::Client>()?;
    m.add_class::<gemini::Response>()?;
    m.add_class::<gemini::PyGemtext>()?;
    m.add_class::<gemini::PyGemtextElement>()?;

    Ok(())
}

#[cfg(feature = "py_bindings")]
use pyo3::wrap_pymodule;
#[cfg(feature = "py_bindings")]
#[pymodule]
fn leda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(gemini))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::gemini::{self, gemtext};

    #[test]
    fn full_test() {
        let url = String::from("gemini://gemini.circumlunar.space/");

        let mut client = gemini::Client::with_timeout(Duration::from_secs(5))
            .expect("Failed to create gemini client");

        let response = client.request(url)
            .expect("Failed to retrieve gemini page");

        let body = &response.body.expect("Body was none!");
        let body = std::str::from_utf8(body)
            .expect("Failed to parse body as utf8");
        let html = gemtext::Gemtext::parse_to_html(body)
            .expect("Failed to parse body as gemtext");

        println!("body:\n{}\n", body);
        println!("html:\n{}\n", html);
    }
}
