pub mod gemini;

#[cfg(test)]
mod tests {
    use crate::gemini::Gemtext;

    use super::gemini;

    #[test]
    fn it_works() {
        let client = gemini::Client::new()
            .expect("Failed to create gemini client");

        let url = String::from("gemini://gemini.circumlunar.space/");
        let response = client.request(url)
            .expect("Failed to retrieve gemini page");
        let body = std::str::from_utf8(&response.body.as_ref().unwrap())
            .expect("Failed to parse body as utf8");
        let _gemtext = Gemtext::new(&body);

        println!("body:{}\n", body)
    }
}
