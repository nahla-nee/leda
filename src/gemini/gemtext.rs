// User facing, we never read from parsed oursevles.
#[allow(dead_code)]
pub struct Gemtext <'a>{
    parsed: Vec<GemtextElement<'a>>
}

enum GemtextElement<'a> {
    Text(&'a str),
    Link(&'a str, Option<&'a str>),
    Heading(&'a str),
    Subheading(&'a str),
    Subsubheading(&'a str),
    UnorederedListItem(&'a str),
    BlockQuote(&'a str),
    Preformatted(&'a str)
}

impl<'a> Gemtext<'a> {
    pub fn new(body: &'a str) -> Gemtext<'a> {
        let mut parsed = Vec::with_capacity(body.lines().count());

        let mut preformatted_mode = false;
        for line in body.lines() {
            if preformatted_mode {
                parsed.push(GemtextElement::Preformatted(line))
            }
            else if line.starts_with("=>") {
                let text = line.split_at(2).1.trim_start();
                if text.is_empty() {
                    // invalid link, just ignore it
                    unimplemented!("Invalid link reached! no url!");
                }

                let (url, text) = if let Some(index) = text.find(char::is_whitespace) {
                    let split = text.split_at(index);
                    (split.0, Some(split.1))
                }
                else {
                    (text, None)
                };

                parsed.push(GemtextElement::Link(url, text));
            }
            else if line.starts_with("###") {
                let text = line.split_at(3).1.trim_start();
                parsed.push(GemtextElement::Subsubheading(text))
            }
            else if line.starts_with("##") {
                let text = line.split_at(2).1.trim_start();
                parsed.push(GemtextElement::Subheading(text))
            }
            else if line.starts_with('#') {
                let text = line.split_at(1).1.trim_start();
                parsed.push(GemtextElement::Heading(text))
            }
            else if line.starts_with('*') {
                let text = line.split_at(1).1.trim_start();
                parsed.push(GemtextElement::UnorederedListItem(text));
            }
            else if line.starts_with('>') {
                let text = line.split_at(1).1.trim_start();
                parsed.push(GemtextElement::BlockQuote(text));
            }
            else if line.starts_with("```") {
                preformatted_mode = !preformatted_mode;
            }
            else {
                parsed.push(GemtextElement::Text(line));
            }
        }

        Gemtext {
            parsed
        }
    }
}
