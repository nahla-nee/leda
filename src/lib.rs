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
//! use leda::gemini::{self, gemtext::Gemtext};
//! 
//! fn main() {
//!     let url = String::from("gemini://gemini.circumlunar.space/")
//! 
//!     let mut client = gemini::Client::builder()
//!         .timeout(Some(Duration::from_secs(5)))
//!         .build()
//!         .expect("Failed to create gemini client");
//! 
//!     let response = client.request(url)
//!         .expect("Failed to retrieve gemini page");
//! 
//!     let body = match &response.header {
//!         gemini::header::StatusCode::Success => &response.body,
//!         // you can handle differents errors, redirects, and input requests as you see fit from
//!         // here on!
//!         _ => panic!("Page requested didn't return a body!")
//!     }
//!     let body = std::str::from_utf8(body)
//!         .expect("Failed to parse body as utf8");
//!     let html = Gemtext::parse_to_html(body)
//!         .expect("Failed to parse body as gemtext");
//! 
//!     println!("raw body: \n{}\n", body);
//!     println!("html body: \n{}\n", html);
//! }
//! ```

pub mod gemini;
#[cfg(feature = "py_bindings")]
pub mod py_bindings;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::gemini::{self, gemtext};

    #[test]
    fn full_test() {
        let url = String::from("gemini://gemini.circumlunar.space/");

        let mut client = gemini::Client::builder()
            .timeout(Some(Duration::from_secs(5)))
            .build()
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
