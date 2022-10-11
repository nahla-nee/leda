use super::header;

/// Represents a response generated from a gemini server.
#[derive(Clone)]
pub struct Response {
    /// The header the server responded with, includes the response status code as well as the meta
    /// information provided.
    pub header: header::Header,
    /// The response body content from the server. `body` will only be `Some` if the header's
    /// [`header::Header::status`] is [`header::StatusCode::Success`], otherwise it'll be `None`.
    pub body: Option<Vec<u8>>,
}

impl Response {
    #[must_use]
    pub fn new(header: header::Header, body: Option<Vec<u8>>) -> Response {
        Response { header, body }
    }
}
