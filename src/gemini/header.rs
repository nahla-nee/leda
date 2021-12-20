use super::error::Error;

pub enum InputCode {
    Input,
    Sensitive
}

pub enum RedirectCode {
    Temporary,
    Permanent
}

pub enum FailTemporaryCode {
    Temporary,
    ServerUnavailable,
    CGIError,
    ProxyError,
    SlowDown,
}

pub enum FailPermanentCode {
    Permanent,
    NotFound,
    Gone,
    ProxyRefused,
    BadRequest
}

pub enum CertFailCode {
    CertRequired,
    CertNotAuthorized,
    CertNotValid
}

pub enum StatusCode {
    Input(InputCode),
    Success,
    Redirect(RedirectCode),
    FailTemporary(FailTemporaryCode),
    FailPermanent(FailPermanentCode),
    CertFail(CertFailCode)
}

pub struct Header {
    pub status: StatusCode,
    pub meta: String
}

impl TryFrom<String> for Header {
    type Error = Error;

    fn try_from(header: String) -> Result<Self, Error> {
        // The proper format of a header is `<STATUS><SPACE><META><CR><LF>`.
        // We must check everything is properly formatted before we interpret any part of it.

        // The checking involves splitting the string, let's make sure it's long enough first.
        // <STATUS> is always 2 integers which is 2 bytes, <SPACE> is defined as 0x20, one byte,
        // <META> must be at least 1 byte, and then <CR> is 1 byte, and <LF> is 1 byte.
        let minimum_length = 6;
        if header.len() < minimum_length {
            return Err(Error::HeaderFormat(
                format!(
                "Header is too small, minimum size is {} bytes but the actual size was {} bytes",
                minimum_length, header.len())
                )
            )
        }

        // See above comment for why this index is 2
        let space_index = 2;
        // Check if space is where it should be and split on it
        let (status, meta) = if header.chars().nth(space_index).unwrap() == ' ' {
            header.split_at(space_index)
        }
        else {
            return Err(Error::HeaderFormat(
                String::from("Failed to find initial whitespace separating <STATUS>
                and <META>, or it isn't where it should be")
            ))
        };

        // The status code must be two integers, no more no less.
        if status.len() != 2 {
            return Err(Error::HeaderFormat(
                format!("The header must be exactly two integers, the header provided ({}), is not",
                status)
            ))
        }
        // The status meta info must end in "\r\n".
        if !meta.ends_with("\r\n") {
            return Err(Error::HeaderFormat(
                String::from("Meta information for the header doesn't end in <CR><LF>")
            ))
        }

        // Remove the CRLF, trim isn't what we want because trailing white space can be part of <META>.
        let meta = &meta[0..meta.len()-2];
        // Header <META> cannot be longer than 1024, if its then the entire header is invalid.
        if meta.len() > 1024 {
            return Err(Error::HeaderFormat(
                format!("The header's meta info was too long, maximum length is 1024 bytes, actual
                length was {} bytes",
                meta.len())
            ))
        }

        let status = StatusCode::from_string(status)?;

        Ok(Header{
            status,
            meta: meta.to_string()
        })
    }
}

impl StatusCode {
    pub fn to_string(&self) -> String {
        match self {
            StatusCode::Input(InputCode::Input) => String::from("10"),
            StatusCode::Input(InputCode::Sensitive) => String::from("11"),
            StatusCode::Success => String::from("20"),
            StatusCode::Redirect(RedirectCode::Temporary) => String::from("30"),
            StatusCode::Redirect(RedirectCode::Permanent) => String::from("31"),
            StatusCode::FailTemporary(FailTemporaryCode::Temporary) => String::from("40"),
            StatusCode::FailTemporary(FailTemporaryCode::ServerUnavailable) => String::from("41"),
            StatusCode::FailTemporary(FailTemporaryCode::CGIError) => String::from("42"),
            StatusCode::FailTemporary(FailTemporaryCode::ProxyError) => String::from("43"),
            StatusCode::FailTemporary(FailTemporaryCode::SlowDown) => String::from("44"),
            StatusCode::FailPermanent(FailPermanentCode::Permanent) => String::from("50"),
            StatusCode::FailPermanent(FailPermanentCode::NotFound) => String::from("51"),
            StatusCode::FailPermanent(FailPermanentCode::Gone) => String::from("52"),
            StatusCode::FailPermanent(FailPermanentCode::ProxyRefused) => String::from("53"),
            StatusCode::FailPermanent(FailPermanentCode::BadRequest) => String::from("59"),
            StatusCode::CertFail(CertFailCode::CertRequired) => String::from("60"),
            StatusCode::CertFail(CertFailCode::CertNotAuthorized) => String::from("61"),
            StatusCode::CertFail(CertFailCode::CertNotValid) => String::from("62")
        }
    }

    pub fn from_string(input: &str) -> Result<StatusCode, Error> {
        Ok(match input {
            "10" => StatusCode::Input(InputCode::Input),
            "11" => StatusCode::Input(InputCode::Sensitive),
            "20" => StatusCode::Success,
            "30" => StatusCode::Redirect(RedirectCode::Temporary),
            "31" => StatusCode::Redirect(RedirectCode::Permanent),
            "40" => StatusCode::FailTemporary(FailTemporaryCode::Temporary),
            "41" => StatusCode::FailTemporary(FailTemporaryCode::ServerUnavailable),
            "42" => StatusCode::FailTemporary(FailTemporaryCode::CGIError),
            "43" => StatusCode::FailTemporary(FailTemporaryCode::ProxyError),
            "44" => StatusCode::FailTemporary(FailTemporaryCode::SlowDown),
            "50" => StatusCode::FailPermanent(FailPermanentCode::Permanent),
            "51" => StatusCode::FailPermanent(FailPermanentCode::NotFound),
            "52" => StatusCode::FailPermanent(FailPermanentCode::Gone),
            "53" => StatusCode::FailPermanent(FailPermanentCode::ProxyRefused),
            "59" => StatusCode::FailPermanent(FailPermanentCode::BadRequest),
            "60" => StatusCode::CertFail(CertFailCode::CertRequired),
            "61" => StatusCode::CertFail(CertFailCode::CertNotAuthorized),
            "62" => StatusCode::CertFail(CertFailCode::CertNotValid),
            _ => return Err(Error::HeaderFormat(
                    format!("Header status code ({}) was not recognized",
                    input)
                ))
        })
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status.to_string(), self.meta)
    }
}