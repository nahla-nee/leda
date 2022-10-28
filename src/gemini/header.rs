use super::Error;

/// Represents the header sent back from a server's response.
#[derive(Clone)]
pub struct Header {
    /// The status code of the response.
    pub status: StatusCode,
    /// The meta information if any is provided. If the server didn't send any additional info
    /// this string will be empty.
    pub meta: String,
}

/// Represents a status code from a server's response header.
#[derive(Clone)]
pub enum StatusCode {
    Input(InputCode),
    Success,
    Redirect(RedirectCode),
    FailTemporary(FailTemporaryCode),
    FailPermanent(FailPermanentCode),
    CertFail(CertFailCode),
}

/// Represents the subtypes of input a server can ask for.
#[derive(Clone)]
pub enum InputCode {
    Input,
    Sensitive,
}

/// Represents the subtypes of redirects a server can ask for.
#[derive(Clone)]
pub enum RedirectCode {
    Temporary,
    Permanent,
}

/// Represents the subtypes of temporary failure a server can have.
#[derive(Clone)]
pub enum FailTemporaryCode {
    Temporary,
    ServerUnavailable,
    CGIError,
    ProxyError,
    SlowDown,
}
/// Represents the subtypes of permanent failure a server can have.
#[derive(Clone)]
pub enum FailPermanentCode {
    Permanent,
    NotFound,
    Gone,
    ProxyRefused,
    BadRequest,
}

/// Represents the subtypes of certificate failure a server can have.
#[derive(Clone)]
pub enum CertFailCode {
    CertRequired,
    CertNotAuthorized,
    CertNotValid,
}

impl TryFrom<String> for Header {
    type Error = Error;

    fn try_from(header: String) -> Result<Self, Error> {
        // The proper format of a header is `<STATUS><SPACE><META><CR><LF>`.
        // We must check everything is properly formatted before we interpret any part of it.

        // The checking involves splitting the string <STATUS> is always 2 integers which is 2
        // bytes, <SPACE> is defined as 0x20, one byte, <META> must be at least 1 byte, and
        // then <CR> is 1 byte, and <LF> is 1 byte.

        let space_index = 2;
        // Check if space is where it should be and split on it
        let (status, meta) = if header.chars().nth(space_index).unwrap() == ' ' {
            // we don't want to split at the header index because then it will include the space in the meta info
            (&header[0..2], &header[3..])
        } else {
            return Err(Error::HeaderFormat(format!(
                "Missing space after status, provided header: {}",
                header
            )));
        };

        // The status code must be two integers, no more no less.
        if status.len() != 2 {
            return Err(Error::HeaderFormat(format!(
                "The status must be exactly two integers, provided header: {}",
                header
            )));
        }
        // The status meta info must end in "\r\n".
        if !meta.ends_with("\r\n") {
            return Err(Error::HeaderFormat(String::from(
                "Meta information for the header doesn't end in <CR><LF>",
            )));
        }

        // Remove the CRLF, trim isn't what we want because trailing white space can be part of <META>.
        let meta = &meta[0..meta.len() - 2];
        // Header <META> cannot be longer than 1024, if its then the entire header is invalid.
        if meta.len() > 1024 {
            return Err(Error::HeaderFormat(format!(
                "The header's meta info was too long, maximum length is 1024 bytes, actual
                length was {} bytes",
                meta.len()
            )));
        }

        let status = StatusCode::from_string(status)?;

        Ok(Header {
            status,
            meta: meta.to_string(),
        })
    }
}

impl StatusCode {
    /// parses a given string and returns its equivalent [`StatusCode`]
    ///
    /// # Errors
    ///
    /// Returns an error if the given string wasn't an exact match to any status code.
    fn from_string(input: &str) -> Result<StatusCode, Error> {
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
            _ => {
                return Err(Error::HeaderFormat(format!(
                    "Header status code ({}) was not recognized",
                    input
                )))
            }
        })
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let val = match self {
            StatusCode::Input(InputCode::Input) => "10",
            StatusCode::Input(InputCode::Sensitive) => "11",
            StatusCode::Success => "20",
            StatusCode::Redirect(RedirectCode::Temporary) => "30",
            StatusCode::Redirect(RedirectCode::Permanent) => "31",
            StatusCode::FailTemporary(FailTemporaryCode::Temporary) => "40",
            StatusCode::FailTemporary(FailTemporaryCode::ServerUnavailable) => "41",
            StatusCode::FailTemporary(FailTemporaryCode::CGIError) => "42",
            StatusCode::FailTemporary(FailTemporaryCode::ProxyError) => "43",
            StatusCode::FailTemporary(FailTemporaryCode::SlowDown) => "44",
            StatusCode::FailPermanent(FailPermanentCode::Permanent) => "50",
            StatusCode::FailPermanent(FailPermanentCode::NotFound) => "51",
            StatusCode::FailPermanent(FailPermanentCode::Gone) => "52",
            StatusCode::FailPermanent(FailPermanentCode::ProxyRefused) => "53",
            StatusCode::FailPermanent(FailPermanentCode::BadRequest) => "59",
            StatusCode::CertFail(CertFailCode::CertRequired) => "60",
            StatusCode::CertFail(CertFailCode::CertNotAuthorized) => "61",
            StatusCode::CertFail(CertFailCode::CertNotValid) => "62",
        };

        write!(f, "{}", val)
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.meta)
    }
}
