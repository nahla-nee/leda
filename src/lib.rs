pub mod gemini;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "py_bindings")]
#[pyfunction]
#[pyo3(name = "gemtext_to_html")]
pub fn py_gemtext_to_html(gemtext: String) -> Result<String, gemini::Error> {
    gemini::util::gemtext_to_html(&gemtext)
}

#[cfg(feature = "py_bindings")]
#[pymodule]
fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gemini::Client>()?;
    m.add_class::<gemini::Response>()?;
    m.add_function(wrap_pyfunction!(py_gemtext_to_html, m)?)?;

    Ok(())
}

#[cfg(feature = "py_bindings")]
use pyo3::wrap_pymodule;
#[cfg(feature = "py_bindings")]
#[pymodule]
fn leda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(gemini))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::gemini::gemtext_to_html;

    use super::gemini;

    #[test]
    fn it_works() {
        let client = gemini::Client::new()
            .expect("Failed to create gemini client");

        let url = String::from("gemini://gemini.circumlunar.space/");
        let response = client.request(url)
            .expect("Failed to retrieve gemini page");

        let body = &response.body.expect("Body was none!");
        let body = std::str::from_utf8(body)
            .expect("Failed to parse body as utf8");
        let gemtext = gemtext_to_html(&body)
            .expect("Failed to parse gemtext");

        println!("body:\n{}\n", body);
        println!("html body:\n{}\n", gemtext);
    }
}
