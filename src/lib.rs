pub mod gemini;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;


#[cfg(feature = "py_bindings")]
#[pymodule]
fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gemini::Client>()?;
    m.add_class::<gemini::Response>()?;
    m.add_class::<gemini::PyGemtext>()?;
    m.add_class::<gemini::PyGemtextElement>()?;

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
    use std::time::Duration;
    use super::gemini;

    #[test]
    fn full_test() {
        let mut client = gemini::Client::with_timeout(Duration::from_secs(5))
            .expect("Failed to create gemini client");

        let url = String::from("gemini://gemini.circumlunar.space/");
        let response = client.request(url)
            .expect("Failed to retrieve gemini page");

        let body = &response.body.expect("Body was none!");
        let body = std::str::from_utf8(body)
            .expect("Failed to parse body as utf8");
        let gemtext = gemini::Gemtext::parse_to_html(body)
            .expect("Failed to parse body as gemtext");

        println!("body:\n{}\n", body);
        println!("html:\n{}\n", gemtext);
    }
}
