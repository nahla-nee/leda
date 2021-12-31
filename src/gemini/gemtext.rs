use super::Error;

// User facing, we never read from parsed oursevles.
#[allow(dead_code)]
pub struct Gemtext <'a>{
    elements: Vec<GemtextElement<'a>>
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

        let mut preformatted_mode = false;
        for line in input.lines() {
            if preformatted_mode {
                elements.push(GemtextElement::Preformatted(line))
            }
            else if let Some(line) = line.strip_prefix("=>") {
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
                preformatted_mode = !preformatted_mode;
                elements.push(GemtextElement::Preformatted(line))
            }
            else {
                elements.push(GemtextElement::Text(line));
            }
        }

        Ok(Gemtext {
            elements
        })
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
#[pyclass]
#[derive(Clone)]
pub struct PyGemtextElement {
    #[pyo3(get)]
    format: u32,
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    link_text: Option<String> // only Some if GemtextElement is a Link
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