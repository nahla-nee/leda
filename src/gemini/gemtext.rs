use core::slice;

use super::Error;

// User facing, we never read from parsed oursevles.
#[allow(dead_code)]
pub struct Gemtext <'a>{
    pub elements: Vec<GemtextElement<'a>>
}

pub enum GemtextElement<'a> {
    Text(&'a str),
    Link(&'a str, &'a str),
    Heading(&'a str),
    Subheading(&'a str),
    Subsubheading(&'a str),
    UnorederedListItem(&'a str),
    BlockQuote(&'a str),
    Preformatted(&'a str)
}

impl<'a> Gemtext<'a> {
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

                elements.push(GemtextElement::Link(url, text));
            }
            else if let Some(line) = line.strip_prefix("###") {
                let text = line.trim_start();
                elements.push(GemtextElement::Subsubheading(text))
            }
            else if let Some(line) = line.strip_prefix("##") {
                let text = line.trim_start();
                elements.push(GemtextElement::Subheading(text))
            }
            else if let Some(line) = line.strip_prefix('#') {
                let text = line.trim_start();
                elements.push(GemtextElement::Heading(text))
            }
            else if let Some(line) = line.strip_prefix('*') {
                let text = line.trim_start();
                elements.push(GemtextElement::UnorederedListItem(text));
            }
            else if let Some(line) = line.strip_prefix('>') {
                let text = line.trim_start();
                elements.push(GemtextElement::BlockQuote(text));
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

                elements.push(GemtextElement::Preformatted(text))
            }
            else {
                elements.push(GemtextElement::Text(line));
            }
        }

        Ok(Gemtext {
            elements
        })
    }

    pub fn parse_to_html(input: &'a str) -> Result<String, Error> {
        let gemtext = Self::new(input)?;
        // This allocation will be a bit too short but should be close enough to only result in
        // one or two reallocations at most
        let mut result = String::with_capacity(input.len());

        let mut elements = gemtext.elements.into_iter().peekable();
        while let Some(element) = elements.next() {
            match element {
                GemtextElement::Text(text) => {
                    result += "<p>";
                    result += text;
                    result += "</p>\n";
                },
                GemtextElement::Link(link, text) => {
                    result += "<a href=\"";
                    result += link;
                    result += "\">";
                    result += text;
                    result += "</a>\n<p></p>\n"
                },
                GemtextElement::Heading(text) => {
                    result += "<h1>";
                    result += text;
                    result += "</h1>\n<p></p>\n";
                },
                GemtextElement::Subheading(text) => {
                    result += "<h2>";
                    result += text;
                    result += "</h2>\n<p></p>\n";
                },
                GemtextElement::Subsubheading(text) => {
                    result += "<h3>";
                    result += text;
                    result += "</h3>\n<p></p>\n";
                },
                GemtextElement::UnorederedListItem(text) => {
                    result += "<ul>\n";

                    result += "<li>";
                    result += text;
                    result += "</li>\n<p></p>\n";
                    while let Some(GemtextElement::UnorederedListItem(item)) = elements.peek() {
                        result += "<li>";
                        result += *item;
                        result += "</li>\n<p></p>\n";

                        elements.next();
                    }

                    result += "</ul>\n<p></p>\n";
                },
                GemtextElement::BlockQuote(text) => {
                    result += "<blockquote>";
                    result += text;
                    result += "</blockquote>\n<p></p>\n";
                },
                GemtextElement::Preformatted(text) => {
                    result += "<pre>";
                    result += text;
                    result += "</pre>\n<p></p>\n";
                },
            }
        }

        Ok(result)
    }
}

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "py_bindings")]
#[pyclass(name = "Gemtext")]
#[derive(Clone)]
pub struct PyGemtext {
    #[pyo3(get)]
    elements: Vec<PyGemtextElement>
}


#[cfg(feature = "py_bindings")]
#[pyclass(name = "GemtextElement")]
#[derive(Clone)]
pub struct PyGemtextElement {
    #[pyo3(get)]
    format: u32, // pyo3 doesn't have great enum representation so python just has to deal with u32
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    link_text: Option<String> // only Some if format is a Link
}

#[cfg(feature = "py_bindings")]
#[pymethods]
impl PyGemtext {
    #[new]
    pub fn new(input: &str) -> Result<PyGemtext, Error> {
        let gemtext = Gemtext::new(input)?;
        let mut elements = Vec::with_capacity(gemtext.elements.len());

        for element in gemtext.elements {
            let values = match element {
                GemtextElement::Text(text) => (0, text.to_string(), None),
                GemtextElement::Link(text, link) => (1, text.to_string(), Some(link.to_string())),
                GemtextElement::Heading(text) => (2, text.to_string(), None),
                GemtextElement::Subheading(text) => (3, text.to_string(), None),
                GemtextElement::Subsubheading(text) => (4, text.to_string(), None),
                GemtextElement::UnorederedListItem(text) => (5, text.to_string(), None),
                GemtextElement::BlockQuote(text) => (6, text.to_string(), None),
                GemtextElement::Preformatted(text) => (7, text.to_string(), None)
            };

            elements.push(PyGemtextElement::new(values.0, values.1, values.2))
        };

        Ok(PyGemtext{
            elements
        })
    }

    #[staticmethod]
    pub fn parse_to_html(input: &str) -> Result<String, Error> {
        Gemtext::parse_to_html(input)
    }
}

#[cfg(feature = "py_bindings")]
impl PyGemtextElement {
    pub fn new(format: u32, value: String, link_text: Option<String>) -> PyGemtextElement {
        PyGemtextElement {
            format,
            value,
            link_text
        }
    }
}