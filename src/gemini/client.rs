use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::response::Response;
use super::header::Header;
use super::error::Error;

use url;

use openssl::ssl;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[cfg_attr(all(feature = "py_bindings"), pyclass)]
pub struct Client {
    connector: ssl::SslConnector,
    timeout: Option<Duration>
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let mut builder = ssl::SslConnector::builder(ssl::SslMethod::tls())
            .map_err(Error::TLSClient)?;
        builder.set_verify(ssl::SslVerifyMode::NONE);
        let connector = builder.build();

        Ok(Client {
            connector,
            timeout: None
        })
    }

    pub fn with_timeout(timeout: Duration) -> Result<Client, Error> {
        let mut client = Self::new()?;
        client.timeout = Some(timeout);

        Ok(client)
    }

    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    pub fn request(&mut self, url: String) -> Result<Response, Error> {
        let (header, body) = self.get_data(url)?;
        let header = Header::try_from(header)?;

        Ok(Response::new(header, body))
    }

    fn get_data(&mut self, mut url: String) -> Result<(String, Option<Vec<u8>>), Error> {
        // Get the proper host string to connect to from the URL.
        let (host, server_name) = {
            let url_parsed = url::Url::parse(&url)
                .map_err(Error::UrlParse)?;
            // We can't use ok_or_else here because that would consume `url` regardless of whether
            // the value is Some or None, and we use url later so it most not be moved.
            let host_str = match url_parsed.host_str() {
                Some(str) => str,
                None => return Err(Error::UrlNoHost(url))
            };
            let port = url_parsed.port().unwrap_or(1965);
    
            (format!("{}:{}", host_str, port), host_str.to_string())
        };

        // Connect to the server and establish a TLS connection.
        let stream = if let Some(timeout) = self.timeout {
            let addrs = host.to_socket_addrs()
                .map_err(Error::TCPConnect)?;
            let mut addrs: Vec<_> = addrs.collect();
            if addrs.is_empty() {
                return Err(Error::UrlNoAddress(host));
            }


            let tail = addrs.pop().unwrap();
            let head = addrs.into_iter()
                .map(|addr| TcpStream::connect_timeout(&addr, timeout))
                .find(|c| c.is_ok());
            if let Some(x) = head {
                x
            } else {
                TcpStream::connect_timeout(&tail, timeout)
            }.map_err(Error::TCPConnect)
        }
        else {
            TcpStream::connect(&host)
                .map_err(Error::TCPConnect)
        }?;
        let mut stream = self.connector.connect(&server_name, stream)
            .unwrap();

        // Check that the URL given to us is proper, the Gemini protocol specifies all URL requests
        // must end in <CR><LF>.
        if !url.ends_with("\r\n") {
            url += "\r\n";
        }

        stream.write(url.as_bytes())
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
            String::from("There must be at least 1 <CR><LF> at the end of the header, but such a \
            sequence was not found.")
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

    #[pyo3(name = "set_timeout")]
    pub fn py_set_timeout(&mut self, seconds: u64) {
        if seconds == 0 {
            self.timeout = None
        }
        else {
            self.timeout = Some(Duration::from_secs(seconds));
        }
    }

    #[pyo3(name = "request")]
    pub fn py_request(&mut self, url: String) -> Result<Response, Error> {
        self.request(url)
    }
}