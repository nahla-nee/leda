use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use super::Error;
use super::header::Header;
use super::response::Response;

use rustls::client::ServerCertVerifier;
use url;

struct NoCertVerification;

impl ServerCertVerifier for NoCertVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

/// Represents a client which will make gemini connections.
pub struct Client {
    tls_config: Arc<rustls::ClientConfig>,
    timeout: Option<Duration>,
}

impl Client {
    /// Creates a client that can be used to make gemini requests
    ///
    /// # Example
    ///
    /// ```
    /// use leda::gemini::Client;
    ///
    /// let client = Client::new().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Will return a [`Error::TLSClient`] if creating a TLS connector failed.
    pub fn new() -> Result<Client, Error> {
        Self::with_timeout(None)
    }

    /// Creates a client that can be used to make gemini requests with a timeout
    /// 
    /// # Example
    /// ```
    /// use leda::gemini::Client;
    /// use std::time::Duration;
    /// 
    /// let client = Client::with_timeout(Some(Duration::new(5, 0)));
    /// ```
    /// 
    /// Will return a [`Error::TLSClient`] if creating a TLS connector failed.
    pub fn with_timeout(timeout: Option<Duration>) -> Result<Client, Error> {
        let tls_config = Arc::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(NoCertVerification))
                .with_no_client_auth(),
        );

        Ok(Client {
            tls_config,
            timeout,
        })
    }

    /// Sets the timeout for the client.
    ///
    /// # Examples
    ///
    /// ```
    /// use leda::gemini::Client;
    /// use std::time::Duration;
    ///
    /// let mut client = Client::new().unwrap();
    /// // A timeout of 5 seconds
    /// client.set_timeout(Some(Duration::from_secs(5)));
    /// // No timeout
    /// client.set_timeout(None);
    /// ```
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Gets the page at `url`.
    ///
    /// The given url must start with the scheme `"gemini://"`
    ///
    /// # Examples
    ///
    /// ```
    /// use leda::gemini::Client;
    ///
    /// let mut client = Client::new().unwrap();
    /// let response = client.request(String::from("gemini://gemini.circumlunar.space/"));
    /// ```
    ///
    /// # Errors
    ///
    /// Will return an [`Error`] if there was a problem with parsing the url, communicating with
    /// the server, or with parsing the servers response.
    pub fn request(&mut self, url: String) -> Result<Response, Error> {
        let (header, body) = self.get_data(url)?;
        let header = Header::try_from(header)?;

        Ok(Response::new(header, body))
    }

    fn get_data(&mut self, mut url: String) -> Result<(String, Option<Vec<u8>>), Error> {
        // Get the proper host string to connect to from the URL.
        let (host, server_name) = {
            let url_parsed = url::Url::parse(&url).map_err(Error::UrlParse)?;
            // We can't use ok_or_else here because that would consume `url` regardless of whether
            // the value is Some or None, and we use url later so it must not be moved.
            let host_str = match url_parsed.host_str() {
                Some(str) => str,
                None => return Err(Error::UrlNoHost(url)),
            };
            let port = url_parsed.port().unwrap_or(1965);

            (format!("{}:{}", host_str, port), host_str.to_string())
        };

        // Connect to the server and establish a TLS connection.
        let rustls_server_name = server_name.as_str().try_into().unwrap();
        let mut conn =
            rustls::ClientConnection::new(self.tls_config.clone(), rustls_server_name).unwrap();

        // Connect, with timeout if requested
        let mut stream = if let Some(timeout) = self.timeout {
            // Get all host addresses so we can attempt to connect to till we get a successful connection
            let mut addresses = host
                .to_socket_addrs()
                .map_err(|e| Error::TCPConnect(e, (&host).clone()))?
                .peekable();
            if addresses.peek().is_none() {
                return Err(Error::UrlNoAddress(host));
            }

            // do this to shut the compiler up, we'll unwrap later because we know it has something in it
            let mut result = None;
            for address in addresses {
                match TcpStream::connect_timeout(&address, timeout) {
                    Ok(r) => {
                        result = Some(Ok(r));
                        break;
                    }
                    Err(e) => result = Some(Err(e)),
                }
            }
            result.unwrap()
        } else {
            TcpStream::connect(host.clone())
        }
        .map_err(|e| Error::TCPConnect(e, (&host).clone()))?;

        let mut tls = rustls::Stream::new(&mut conn, &mut stream);

        // Check that the URL given to us is proper, the Gemini protocol specifies all URL requests
        // must end in <CR><LF>.
        if !url.ends_with("\r\n") {
            url += "\r\n";
        }

        tls.write(url.as_bytes())
            .map_err(|e| Error::StreamIO("Failed to send request to server", e))?;

        // We can't parse this as a string yet, we can be confident-ish that the header is UTF-8,
        // but we have no idea what the body is.
        let mut response = Vec::new();
        tls.read_to_end(&mut response)
            .map_err(|e| Error::StreamIO("Failed to read resposne from server", e))?;

        // The Gemini protocol specifies that the response must have a header, and optionally a body
        // which are separated by <CR><LF>. <CR><LF> must be there regardless of if a
        // body exists.
        let header_cutoff = {
            let mut cutoff = None;
            for i in 0..(response.len() - 1) {
                if &response[i..=(i + 1)] == "\r\n".as_bytes() {
                    cutoff = Some(i + 2);
                    break;
                }
            }

            cutoff
        }
        .ok_or_else(|| {
            Error::HeaderFormat(String::from(
                "There must be at least 1 <CR><LF> at the end of the header, but such a \
            sequence was not found.",
            ))
        })?;

        let (header, body) = response.split_at(header_cutoff);
        let header = String::from_utf8_lossy(header).to_string();
        // Even if a body doesn't exist, rust will return an empty string for the body, we should
        // check then if a body does or doesn't exist by checking if the body string is empty.
        let body = if body.is_empty() {
            None
        } else {
            Some(body.to_vec())
        };

        Ok((header, body))
    }
}
