use leda::gemini::{self, gemtext::Gemtext};
use std::time::Duration;

fn main() {
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

    let body = std::str::from_utf8(&body).expect("Failed to parse body as utf8");
    assert!(Gemtext::new(body).is_ok());
    println!("raw body: \n{}\n", body);
}
