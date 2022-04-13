use leda::gemini::{self, gemtext::Gemtext};
use std::time::Duration;

fn main() {
    let url = String::from("gemini://gemini.circumlunar.space/");

    let mut client = gemini::Client::builder()
        .timeout(Some(Duration::from_secs(5)))
        .build()
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
    let html = Gemtext::new(body)
        .expect("Failed to parse body as gemtext")
        .to_html();

    println!("raw body: \n{}\n", body);
    println!("html body: \n{}\n", html);
}
