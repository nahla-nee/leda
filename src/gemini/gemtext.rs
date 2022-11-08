use super::Error;

/// Represents a gemtext document by element, line by line.
#[derive(Debug, PartialEq)]
pub struct Gemtext {
    /// List of elements.
    pub elements: Vec<Element>,
}

/// Represents the varying elements a gemtext document can have.
#[derive(Debug, PartialEq)]
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
    UnorderedList(Vec<String>),
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
    ///                    =>gemini://gemini.circumlunar.space/ gemini homepage link";
    /// let parsed_doc = Gemtext::new(example_doc)
    ///     .expect("Failed to parse gemtext document");
    /// // Notice that the space between a format specifier and the text it specifies matters!
    /// let expected_result = [
    ///                         gemtext::Element::Heading(String::from(" Example gemtext header")),
    ///                         gemtext::Element::Text(String::from("I'm a paragraph!")),
    ///                         gemtext::Element::Link(String::from("gemini://gemini.circumlunar.space/"), String::from("gemini homepage link"))
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

        // we have to de-sugar what would be a for loop into a while loop
        // because of how we parse 
        println!("INPUT:\n{}", input);
        let mut lines = input.lines().enumerate().peekable();
        while let Some((index, line)) = lines.next() {
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
                    (split.0.trim(), split.1)
                } else {
                    (text, text)
                };

                elements.push(Element::Link(url.to_string(), text.to_string()));
            } else if let Some(line) = line.strip_prefix("###") {
                elements.push(Element::Subsubheading(line.to_string()));
            } else if let Some(line) = line.strip_prefix("##") {
                elements.push(Element::Subheading(line.to_string()));
            } else if let Some(line) = line.strip_prefix('#') {
                elements.push(Element::Heading(line.to_string()));
            } else if let Some(line) = line.strip_prefix('*') {
                let mut list = Vec::new();

                list.push(line.to_string());

                while let Some((_idx, line)) = lines.peek() {
                    if let Some(line) = line.strip_prefix('*') {
                        list.push(line.to_string());
                        lines.next();
                    } else {
                        break;
                    }
                }

                elements.push(Element::UnorderedList(list));
            } else if let Some(line) = line.strip_prefix('>') {
                elements.push(Element::BlockQuote(line.to_string()));
            } else if let Some(line) = line.strip_prefix("```") {
                let alt_text = line.to_string();
                let mut preformatted_block = String::new();

                while let Some((_idx, line)) = lines.peek() {
                    if let None = line.strip_prefix("```") {
                        preformatted_block += line;
                        preformatted_block += "\n";
                        lines.next();
                    }
                    else {
                        // skip the ending ```
                        lines.next();
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
