use super::Error;

/// Represents a gemtext document by element, line by line.
#[derive(Debug)]
pub struct Gemtext {
    /// List of elements.
    pub elements: Vec<Element>,
}

/// Represents the varying elements a gemtext document can have.
#[derive(Debug, PartialEq, Eq)]
pub enum Element {
    /// Text without any specific formatting, to be treated like a paragraph
    Text(String),
    /// A link, the first member of the tuple is where the link goes to, the
    /// second member is the human readable text to display for this link.
    Link(String, String),
    /// Header
    Heading(String),
    /// Sub-header
    Subheading(String),
    /// Sub-sub-header
    Subsubheading(String),
    /// An unoredered list item, these will appear in the order they showed up in.
    UnorderedListItem(String),
    /// A block quote
    BlockQuote(String),
    /// The first element is the alt text, the second element is the preformatted text
    Preformatted(String, String),
}

impl<'a> Gemtext {
    /// Creates a new [`Gemtext`] document from the given string.
    ///
    /// # Examples
    ///
    /// ```
    /// use leda::gemini::gemtext::{self, Gemtext};
    ///
    /// let example_doc = "# Example gemtext header\n\
    ///                    I'm a paragraph!\n\
    ///                    => gemini://gemini.circumlunar.space/ gemini homepage link";
    /// let parsed_doc = Gemtext::new(example_doc)
    ///     .expect("Failed to parse gemtext document");
    /// let expected_result = [
    ///                        gemtext::Element::Heading(String::from("Example gemtext header")),
    ///                        gemtext::Element::Text(String::from("I'm a paragraph!")),
    ///                        gemtext::Element::Link(String::from("gemini homepage link"), String::from("gemini://gemini.circumlunar.space/"))
    ///                       ];
    /// for (real, expected) in parsed_doc.elements.iter().zip(expected_result.iter()) {
    ///     assert_eq!(real, expected);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Will return an [`Error::GemtextFormat`] if there was a problem with parsing the document.
    pub fn new(input: &'a str) -> Result<Gemtext, Error> {
        let mut elements = Vec::with_capacity(input.lines().count());

        let mut lines = input.lines().peekable();
        for (index, line) in input.lines().enumerate() {
            if let Some(line) = line.strip_prefix("=>") {
                let text = line.trim_start();
                if text.is_empty() {
                    // invalid link has no value.
                    return Err(Error::GemtextFormat(format!(
                        "Invalid link format, there must be \
                        something after =>. Line #{}: {}",
                        index + 1,
                        line.trim()
                    )));
                }

                let (url, text) = if let Some(index) = text.find(char::is_whitespace) {
                    // get rid of the first space character, if there's more space then its part of
                    // how the human readable text is formatted.
                    let split = text.split_at(index+1);
                    (split.0.trim_end(), split.1)
                } else {
                    (text, text)
                };

                elements.push(Element::Link(url.to_string(), text.to_string()));
            } else if let Some(line) = line.strip_prefix("###") {
                elements.push(Element::Subsubheading(line.to_string()));
            } else if let Some(line) = line.strip_prefix("##") {
                elements.push(Element::Subheading(line.to_string()));
            } else if let Some(line) = line.strip_prefix('#') {
                let text = line.trim_start();
                elements.push(Element::Heading(text.to_string()));
            } else if let Some(line) = line.strip_prefix('*') {
                elements.push(Element::UnorderedListItem(line.to_string()));
            } else if let Some(line) = line.strip_prefix('>') {
                elements.push(Element::BlockQuote(line.to_string()));
            } else if let Some(line) = line.strip_prefix("```") {
                let alt_text = line.to_string();
                let mut preformatted_block = String::new();

                while let Some(line) = lines.peek() {
                    if let None = line.strip_prefix("```") {
                        preformatted_block += line;
                        lines.next();
                    }
                    else {
                        break;
                    }
                }

                elements.push(Element::Preformatted(alt_text, preformatted_block));
            } else {
                elements.push(Element::Text(line.to_string()));
            }
        }

        Ok(Gemtext { elements })
    }
}
