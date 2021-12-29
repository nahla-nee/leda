use super::Error;

pub fn gemtext_to_html(gemtext: &str) -> Result<String, Error> {
    let mut parsed = String::with_capacity(gemtext.len());

    let mut preformatted_mode = false;
    // we have to put list items in a `<ul>`, use this to check if we already added the tag or not
    let mut in_list = false;
    for line in gemtext.lines() {
        if preformatted_mode {
            parsed += line;
            parsed += "\n";
            continue;
        }

        // we have to place this first and make it a separate if block because we have to end the
        // list if we already started it and the new line isnt a list item
        if let Some(line) = line.strip_prefix('*') {
            if !in_list {
                parsed += "<ul>\n";
                in_list = true;
            }

            let text = line.trim_start();
            parsed += &format!("<li>{}</li>\n", text);
            continue;
        }
        else if in_list {
            parsed += "</ul>\n";
            in_list = false;
        }

        if let Some(line) = line.strip_prefix("=>") {
            let text = line.trim_start();
            if text.is_empty() {
                return Err(Error::GemtextFormat(format!("Invalid link format, there must be something \
                    after =>. Line: {}", line.trim())));
            }

            let (url, text) = if let Some(index) = text.find(char::is_whitespace) {
                let (url, text) = text.split_at(index);
                (url, text.trim_start())
            }
            else {
                (text, text)
            };

            parsed += &format!("<a href=\"{}\">{}</a>", url, text);
        }
        else if let Some(line) = line.strip_prefix("###") {
            let text = line.trim_start();
            parsed += &format!("<h3>{}</h3>", text);
        }
        else if let Some(line) = line.strip_prefix("##") {
            let text = line.trim_start();
            parsed += &format!("<h2>{}</h2>", text);
        }
        else if let Some(line) = line.strip_prefix('#') {
            let text = line.trim_start();
            parsed += &format!("<h1>{}</h1>", text);
        }
        else if let Some(line) = line.strip_prefix('>') {
            let text = line.trim_start();
            parsed += &format!("<blockquote>{}</blockquote>", text);
        }
        else if line.starts_with("```") {
            preformatted_mode = !preformatted_mode;

            if preformatted_mode {
                parsed += "<pre>\n";
                parsed += line;
            }
            else {
                parsed += "</pre>"
            }
        }
        else {
            parsed += &format!("<p>{}</p>", line);
        }

        parsed += "\n";
    }

    parsed.shrink_to_fit();
    Ok(parsed)
}