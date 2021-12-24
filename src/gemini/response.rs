use super::Header;

#[derive(Clone)]
pub struct Response {
    pub header: Header,
    pub body: Option<Vec<u8>>
}

impl Response {
    pub fn new(header: Header, body: Option<Vec<u8>>) -> Response {
        Response {
            header,
            body
        }
    }
}