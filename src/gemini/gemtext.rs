use core::slice;

use super::Error;

/// Represents a gemtext document by element, line by line.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Gemtext <'a>{
    pub elements: Vec<Element<'a>>
}

/// Represents the varying elements a gemtext document can have.
#[derive(Debug, PartialEq, Eq)]
pub enum Element<'a> {
    /// Text without any specific formatting, to be treated like a paragraph
    Text(&'a str),
    /// A link, the first member of the tuple is where the link goes to, the second member is the
    /// human readable text to display for this link.
    Link(&'a str, &'a str),
    /// Header
    Heading(&'a str),
    /// Sub-header 
    Subheading(&'a str),
    /// Sub-sub-header
    Subsubheading(&'a str),
    /// An item in unordered list
    UnorederedListItem(&'a str),
    /// A block quote
    BlockQuote(&'a str),
    /// An unspecified number of lines that have been preformatted.
    Preformatted(&'a str)
}

impl<'a> Gemtext<'a> {
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
    ///                        gemtext::Element::Heading("Example gemtext header"),
    ///                        gemtext::Element::Text("I'm a paragraph!"),
    ///                        gemtext::Element::Link("gemini://gemini.circumlunar.space/",
    ///                             "gemini homepage link")
    ///                       ];
    /// for (real, expected) in parsed_doc.elements.iter().zip(expected_result.iter()) {
    ///     assert_eq!(real, expected);
    /// }
    /// ```
    /// 
    /// # Errors
    /// 
    /// Will return an [`Error::GemtextFormat`] if there was a problem with parsing the document.
    pub fn new(input: &'a str) -> Result<Gemtext<'a>, Error> {
        let mut elements = Vec::with_capacity(input.lines().count());

        let mut lines = input.lines();
        while let Some(line) = lines.next() {
            if let Some(line) = line.strip_prefix("=>") {
                let text = line.trim_start();
                if text.is_empty() {
                    // invalid link has no value.
                    return Err(Error::GemtextFormat(format!("Invalid link format, there must be \
                        something after =>. Line: {}", line.trim())));
                }

                let (url, text) = if let Some(index) = text.find(char::is_whitespace) {
                    let split = text.split_at(index);
                    (split.0, split.1.trim_start())
                }
                else {
                    (text, text)
                };

                elements.push(Element::Link(url, text));
            }
            else if let Some(line) = line.strip_prefix("###") {
                let text = line.trim_start();
                elements.push(Element::Subsubheading(text));
            }
            else if let Some(line) = line.strip_prefix("##") {
                let text = line.trim_start();
                elements.push(Element::Subheading(text));
            }
            else if let Some(line) = line.strip_prefix('#') {
                let text = line.trim_start();
                elements.push(Element::Heading(text));
            }
            else if let Some(line) = line.strip_prefix('*') {
                let text = line.trim_start();
                elements.push(Element::UnorederedListItem(text));
            }
            else if let Some(line) = line.strip_prefix('>') {
                let text = line.trim_start();
                elements.push(Element::BlockQuote(text));
            }
            else if let Some(line) = line.strip_prefix("```") {
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

                elements.push(Element::Preformatted(text));
            }
            else {
                elements.push(Element::Text(line));
            }
        }

        Ok(Gemtext {
            elements
        })
    }

    /// Creates an html [`String`] to represent the given gemtext document
    /// 
    /// # Examples
    /// 
    /// ```
    /// use leda::gemini::gemtext;
    /// 
    /// let example_doc = "# Example raw gemtext header\n\
    ///                    I'm a paragraph!\n\
    ///                    => gemini://gemini.circumlunar.space/ gemini homepage link";
    /// let parsed_doc = gemtext::Gemtext::parse_to_html(example_doc).unwrap();
    /// let expected_result = concat!(
    ///                        "<h1>Example raw gemtext header</h1>\n",
    ///                        "<p></p>\n",
    ///                        "<p>I'm a paragraph!</p>\n",
    ///                        "<a href=\"gemini://gemini.circumlunar.space/\">gemini homepage link</a>\n"
    ///                        "<p></p>\n");
    /// assert_eq!(expected_result, parsed_doc)
    /// ```
    /// 
    /// # Errors
    /// 
    /// Will return an [`Error::GemtextFormat`] if there was a problem with parsing the document.
    pub fn parse_to_html(input: &'a str) -> Result<String, Error> {
        let gemtext = Self::new(input)?;
        // This allocation will be a bit too short but should be close enough to only result in
        // one or two reallocations at most
        let mut result = String::with_capacity(input.len());

        let mut elements = gemtext.elements.into_iter().peekable();
        while let Some(element) = elements.next() {
            match element {
                Element::Text(text) => {
                    result += "<p>";
                    result += text;
                    result += "</p>\n";
                },
                Element::Link(link, text) => {
                    result += "<a href=\"";
                    result += link;
                    result += "\">";
                    result += text;
                    result += "</a>\n<p></p>\n";
                },
                Element::Heading(text) => {
                    result += "<h1>";
                    result += text;
                    result += "</h1>\n<p></p>\n";
                },
                Element::Subheading(text) => {
                    result += "<h2>";
                    result += text;
                    result += "</h2>\n<p></p>\n";
                },
                Element::Subsubheading(text) => {
                    result += "<h3>";
                    result += text;
                    result += "</h3>\n<p></p>\n";
                },
                Element::UnorederedListItem(text) => {
                    result += "<ul>\n";

                    result += "<li>";
                    result += text;
                    result += "</li>\n<p></p>\n";
                    while let Some(Element::UnorederedListItem(item)) = elements.peek() {
                        result += "<li>";
                        result += *item;
                        result += "</li>\n<p></p>\n";

                        elements.next();
                    }

                    result += "</ul>\n<p></p>\n";
                },
                Element::BlockQuote(text) => {
                    result += "<blockquote>";
                    result += text;
                    result += "</blockquote>\n<p></p>\n";
                },
                Element::Preformatted(text) => {
                    result += "<pre>";
                    result += text;
                    result += "</pre>\n<p></p>\n";
                },
            }
        }

        Ok(result)
    }
}