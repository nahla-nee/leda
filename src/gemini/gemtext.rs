use core::slice;

use super::Error;

/// Represents a gemtext document by element, line by line.
#[derive(Debug)]
pub struct Gemtext {
    /// List of elements.
    pub elements: Vec<Element>
}

/// Represents the varying elements a gemtext document can have.
#[derive(Debug, PartialEq, Eq)]
pub enum Element {
    /// Text without any specific formatting, to be treated like a paragraph
    Text(String),
    /// A link, the first member of the tuple is the human readable text to
    /// display for this link, the second member is where the link goes to .
    Link(String, String),
    /// Header
    Heading(String),
    /// Sub-header
    Subheading(String),
    /// Sub-sub-header
    Subsubheading(String),
    /// An unoredered list, each item in the vector is a list item.
    UnorderedList(Vec<String>),
    /// A block quote
    BlockQuote(String),
    /// An unspecified number of lines that have been preformatted.
    Preformatted(String),
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
    ///                        gemtext::Element::Link(String::from("gemini://gemini.circumlunar.space/"),
    ///                             String::from("gemini homepage link"))
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
                        index+1, line.trim()
                    )));
                }

                let (text, url) = if let Some(index) = text.find(char::is_whitespace) {
                    let split = text.split_at(index);
                    (split.0, split.1.trim_start())
                } else {
                    (text, text)
                };

                elements.push(Element::Link(url.to_string(), text.to_string()));
            } else if let Some(line) = line.strip_prefix("###") {
                let text = line.trim_start();
                elements.push(Element::Subsubheading(text.to_string()));
            } else if let Some(line) = line.strip_prefix("##") {
                let text = line.trim_start();
                elements.push(Element::Subheading(text.to_string()));
            } else if let Some(line) = line.strip_prefix('#') {
                let text = line.trim_start();
                elements.push(Element::Heading(text.to_string()));
            } else if let Some(line) = line.strip_prefix('*') {
                let mut list = Vec::new();

                let text = line.trim_start();
                list.push(text.to_string());
                
                // Can't use for loop here because it would consume the iterator.
                while let Some(line) = lines.peek() {
                    if let Some(line) = line.strip_prefix('*') {
                        let text = line.trim_start();
                        list.push(text.to_string());

                        lines.next();
                    } else {
                        break;
                    }
                }

                elements.push(Element::UnorderedList(list));
            } else if let Some(line) = line.strip_prefix('>') {
                let text = line.trim_start();
                elements.push(Element::BlockQuote(text.to_string()));
            } else if let Some(line) = line.strip_prefix("```") {
                let start = line.as_ptr();
                let mut len = line.len();

                for line in lines.by_ref() {
                    if line.strip_prefix("```").is_some() {
                        break;
                    }

                    len += line.len();
                }

                let text = unsafe {
                    let str_slice = slice::from_raw_parts(start, len);
                    std::str::from_utf8_unchecked(str_slice)
                };

                elements.push(Element::Preformatted(text.to_string()));
            } else {
                elements.push(Element::Text(line.to_string()));
            }
        }

        Ok(Gemtext {
            elements,
        })
    }
}
