use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use super::response::Response;
use super::header::Header;
use super::error::Error;

use rustls;
use rustls_native_certs;
use url;

#[cfg(feature = "py_bindings")]
use pyo3::prelude::*;

#[cfg_attr(all(feature = "py_bindings"), pyclass)]
pub struct Client {
    client_config: Arc<rustls::ClientConfig>,
    timeout: Option<Duration>
}

struct NoCertVerifier {}

impl rustls::client::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _: &rustls::Certificate,
        _: &[rustls::Certificate],
        _: &rustls::ServerName,
        _: &mut dyn Iterator<Item = &[u8]>,
        _: &[u8],
        _: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}


impl Client {
    pub fn new() -> Result<Client, Error> {
        let mut root_store = rustls::RootCertStore::empty();
        let native_roots = rustls_native_certs::load_native_certs()
            .map_err(Error::TLSClient)?;
        for cert in native_roots {
            root_store.add(&rustls::Certificate(cert.0))
                .map_err(Error::TLSCert)?;
        }

        let cert_verifier = Arc::new(NoCertVerifier {});

        let mut client_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        client_config.dangerous().set_certificate_verifier(cert_verifier);

        let client_config = Arc::new(client_config);

        Ok(Client {
            client_config,
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
        let server_name = (&server_name as &str).try_into().unwrap();
        let mut tls_conn = rustls::ClientConnection::new(self.client_config.clone(), server_name)
            .expect("failed to connect");
        let mut stream = if let Some(timeout) = self.timeout {
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

        let mut tls_client = rustls::Stream::new(&mut tls_conn, &mut stream);

        // Check that the URL given to us is proper, the Gemini protocol specifies all URL requests
        // must end in <CR><LF>.
        if !url.ends_with("\r\n") {
            url += "\r\n";
        }

        tls_client.write(url.as_bytes())
            .map_err(|e| Error::StreamIO("Failed to send request to server", e))?;
    
        // We can't parse this as a string yet, we can be confident-ish that the header is UTF-8,
        // but we have no idea what the body is.
        let mut response = Vec::new();
        tls_client.read_to_end(&mut response)
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