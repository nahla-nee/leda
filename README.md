# Leda

A crate that implements the client logic for several small internet protocols. Currently only
supports gemini but with plans to support other protocols such as finger, and gopher.

## Cargo Features

- **py_bindings**: generates bindings for python

## Get started

This is a minimal example to show what using this library is like.

```rs
use leda::gemini;

fn main() {
    let url = String::from("gemini://gemini.circumlunar.space/")

    let mut client = gemini::Client::with_timeout(Duration::from_secs(5))
        .expect("Failed to create gemini client");

    let response = client.request(url)
        .expect("Failed to retrieve gemini page");

    let body = match &response.header {
        gemini::header::StatusCode::Success => &response.body,
        // you can handle differents errors, redirects, and input requests as you see fit from
        // here on!
        _ => panic!("Page requested didn't return a body!")
    }
    let body = std::str::from_utf8(body)
        .expect("Failed to parse body as utf8");
    let html = gemini::gemtext::Gemtext::parse_to_html(body)
        .expect("Failed to parse body as gemtext");

    println!("raw body: \n{}\n", body);
    println!("html body: \n{}\n", html);
}
```