pub mod gemini;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "py_bindings")]
#[pymodule]
fn gemini(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gemini::Client>()?;
    m.add_class::<gemini::Response>()?;

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
    use crate::gemini::Gemtext;

    use super::gemini;

    #[test]
    fn it_works() {
        let client = gemini::Client::new()
            .expect("Failed to create gemini client");

        let url = String::from("gemini://gemini.circumlunar.space/");
        let response = client.request(url)
            .expect("Failed to retrieve gemini page");
        let body = std::str::from_utf8(&response.body.as_ref().unwrap())
            .expect("Failed to parse body as utf8");
        let _gemtext = Gemtext::new(&body);

        println!("body:{}\n", body)
    }
}
