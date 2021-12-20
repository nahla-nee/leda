use std::net::TcpStream;
use std::io::{Write, Read};

use super::response::Response;
use super::header::Header;
use super::error::Error;

use native_tls::TlsConnector;
use url;

pub struct Client {
    connector: TlsConnector
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let connector = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .or_else(|e| Err(Error::TLSConnector(e)))?;

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
                .or_else(|e| Err(Error::UrlParse(e)))?;
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
            .or_else(|e| Err(Error::TCPConnect(e)))?;
        let mut stream = self.connector.connect(&host, stream)
            .or_else(|e| Err(Error::TLSHandshake(e)))?;

        // Check that the URL given to us is proper, the Gemini protocol specifies all URL requests
        // must end in <CR><LF>.
        if !url.ends_with("\r\n") {
            url = url + "\r\n";
        }

        stream.write_all(url.as_bytes())
            .or_else(|e| Err(Error::StreamIO("Failed to send request to server", e)))?;
    
        // We can't parse this as a string yet, we can be confident-ish that the header is UTF-8,
        // but we have no idea what the body is.
        let mut response = Vec::new();
        stream.read_to_end(&mut response)
            .or_else(|e| Err(Error::StreamIO("Failed to read resposne from server", e)))?;


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
        }.ok_or(Error::HeaderFormat(
            String::from("There must be at least 1 <CR><LF> at the end of the header, but such a
            sequence was not found not found.")
        ))?;

        let (header, body) = response.split_at(header_cutoff);
        let header = String::from_utf8_lossy(&header).to_string();
        // Even if a body doesn't exist, rust will return an empty string for the body, we should
        // check then if a body does or doesn't exist by checking if the body string is empty.
        let body = if body.is_empty() {
            None
        }
        else {
            Some(body.to_vec())
        };

        stream.shutdown().or_else(|e| Err(Error::StreamIO("Failed to shutdown TCP stream", e)))?;

        Ok((header, body))
    }
}
