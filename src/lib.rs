//! A crate that implements the client logic for several small internet protocols. Currently only
//! supports gemini but with plans to support other protocols such as finger, and gopher.
//! 
//! ## Get started
//!
//! This is a minimal example to show what using this library is like.
//!
//! ```no_run
//! use leda::gemini::{self, gemtext::Gemtext};
//! use std::time::Duration;
//!
//! let url = String::from("gemini://gemini.circumlunar.space/");
//!
//! let mut client = gemini::Client::with_timeout(Some(Duration::from_secs(5)))
//!     .expect("Failed to create gemini client");
//!
//! let response = client.request(url)
//!     .expect("Failed to retrieve gemini page");
//!
//! // Check that the server responded successfully with a gemtext document
//! let body = if let gemini::header::StatusCode::Success = response.header.status {
//!     if !response.header.meta.starts_with("text/gemini") {
//!         panic!("The server didn't respond with a gemtext document when we expected it to");
//!     }
//!     response.body.as_ref().unwrap()
//! }
//! else {
//!     // you can handle differents errors, redirects, and input requests as you see fit from
//!     // here on!
//!     panic!("Page requested didn't return a body!");
//! };
//!
//! let body = std::str::from_utf8(&body)
//!     .expect("Failed to parse body as utf8");
//!
//! println!("raw body: \n{}\n", body);
//! ```

pub mod gemini;

#[cfg(test)]
mod tests {
    use super::gemini::{self, gemtext::Gemtext};
    use std::time::Duration;

    #[test]
    fn request_test() {
        let url = String::from("gemini://gemini.circumlunar.space/");

        let mut client = gemini::Client::with_timeout(Some(Duration::from_secs(5)))
            .expect("Failed to create gemini client");

        let response = client.request(url).expect("Failed to retrieve gemini page");

        // Check that the server responded successfully with a gemtext document
        let body = if let gemini::header::StatusCode::Success = response.header.status {
            if !response.header.meta.starts_with("text/gemini") {
                panic!("The server didn't respond with a gemtext document when we expected it to");
            }
            response.body.as_ref().unwrap()
        } else {
            // you can handle differents errors, redirects, and input requests as you see fit from
            // here on!
            panic!("Page requested didn't return a body!");
        };

        let body = std::str::from_utf8(body).expect("Failed to parse body as utf8");
        assert!(Gemtext::new(body).is_ok());
        println!("body:\n{}\n", body);
    }

    #[test]
    fn gemtext_parse_test() {
        let gemtext_src = "paragraph\n\
            => gemini:://gemini.circumlunar.space/ link test\n\
            # Heading\n\
            ## Sub-heading\n\
            ### Sub-sub-heading\n\
            *list\n\
            *example\n\
            > blockquote\n\
            ```\n\
            ___________________________________\n\
            |                                 |\n\
            | This is some pre formatted text |\n\
            |_________________________________|\n\
            ```";
        let expected_parse = [
            gemini::gemtext::Element::Text("paragraph".to_string()),
            gemini::gemtext::Element::Link("gemini:://gemini.circumlunar.space/".to_string(), "link test".to_string()),
            gemini::gemtext::Element::Heading(" Heading".to_string()),
            gemini::gemtext::Element::Subheading(" Sub-heading".to_string()),
            gemini::gemtext::Element::Subsubheading(" Sub-sub-heading".to_string()),
            gemini::gemtext::Element::UnorderedList(vec!["list".to_string(), "example".to_string()]),
            gemini::gemtext::Element::BlockQuote(" blockquote".to_string()),
            gemini::gemtext::Element::Preformatted("".to_string(), "___________________________________\n\
                                                                    |                                 |\n\
                                                                    | This is some pre formatted text |\n\
                                                                    |_________________________________|\n".to_string())
        ];

        let result = gemini::Gemtext::new(gemtext_src)
            .expect("Failed to parse gemtext_src");
        assert_eq!(result.elements, expected_parse);
    }
}
