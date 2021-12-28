use std::net::TcpStream;
use std::io::{Write, Read};

use super::response::Response;
use super::header::Header;
use super::error::Error;

use native_tls::TlsConnector;
use url;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[derive(Clone)]
#[cfg_attr(all(feature = "py_bindings"), pyclass())]
pub struct Client {
    connector: TlsConnector
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let connector = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build().map_err(Error::TLSConnector)?;

        Ok(Client{
            connector
        })
    }

    pub fn request(&self, url: String) -> Result<Response, Error> {
        let (header, body) = self.get_data(url)?;
        let header = Header::try_from(header)?;

        Ok(Response::new(header, body))
    }

    fn get_data(&self, mut url: String) -> Result<(String, Option<Vec<u8>>), Error> {
        // Get the proper host string to connect to from the URL.
        let host = {
            let url_parsed = url::Url::parse(&url)
                .map_err(Error::UrlParse)?;
            // We can't use ok_or_else here because that would consume `url` regardless of whether
            // the value is Some or None, and we use url later so it most not be moved.
            let host_str = match url_parsed.host_str() {
                Some(str) => str,
                None => return Err(Error::UrlNoHost(url))
            };
            let port = url_parsed.port().unwrap_or(1965);
    
            format!("{}:{}", host_str, port)
        };

        // Connect to the server and establish a TLS connection.
        let stream = TcpStream::connect(&host)
            .map_err(Error::TCPConnect)?;
        let mut stream = self.connector.connect(&host, stream)
            .map_err(|e| Error::TLSHandshake(Box::new(e)))?;

        // Check that the URL given to us is proper, the Gemini protocol specifies all URL requests
        // must end in <CR><LF>.
        if !url.ends_with("\r\n") {
            url += "\r\n";
        }

        stream.write_all(url.as_bytes())
            .map_err(|e| Error::StreamIO("Failed to send request to server", e))?;
    
        // We can't parse this as a string yet, we can be confident-ish that the header is UTF-8,
        // but we have no idea what the body is.
        let mut response = Vec::new();
        stream.read_to_end(&mut response)
            .map_err(|e| Error::StreamIO("Failed to read resposne from server", e))?;


        // The Gemini protocol specifies that the response must have a header, and optionally a body
        // which are separated by <CR><LF>. <CR><LF> must be there regardless of if a
        // body exists.
        let header_cutoff = {
            let mut cutoff = None;
            for i in 0..(response.len()-1) {
                if &response[i..=(i+1)] == "\r\n".as_bytes() {
                    cutoff = Some(i+2);
                    break;
                }
            }

            cutoff
        }.ok_or_else(|| Error::HeaderFormat(
            String::from("There must be at least 1 <CR><LF> at the end of the header, but such a
            sequence was not found not found.")
        ))?;

        let (header, body) = response.split_at(header_cutoff);
        let header = String::from_utf8_lossy(header).to_string();
        // Even if a body doesn't exist, rust will return an empty string for the body, we should
        // check then if a body does or doesn't exist by checking if the body string is empty.
        let body = if body.is_empty() {
            None
        }
        else {
            Some(body.to_vec())
        };

        stream.shutdown().map_err(|e| Error::StreamIO("Failed to shutdown TCP stream", e))?;

        Ok((header, body))
    }
}

#[cfg(feature = "py_bindings")]
#[pymethods]
impl Client {
    #[new]
    pub fn __new__() -> Result<Client, Error> {
        Client::new()
    }

    #[pyo3(name = "request")]
    pub fn py_request(&self, url: String) -> Result<Response, Error> {
        self.request(url)
    }
}