use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Ron(ron::Error),
    RonParse(ron::error::SpannedError),
    Http(ureq::Error),
    Json(serde_json::Error),
    Service(String),
    InvalidProfileName(String),
    ProfileNotFound(String),
    ProfileExists(String),
    NoDefaultProfile,
    NoConfigHome,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Ron(e) => write!(f, "ron error: {e}"),
            Error::RonParse(e) => write!(f, "failed to parse RON: {e}"),
            Error::Http(e) => write!(f, "http error: {e}"),
            Error::Json(e) => write!(f, "json error: {e}"),
            Error::Service(msg) => write!(f, "service error: {msg}"),
            Error::InvalidProfileName(name) => {
                write!(f, "invalid profile name '{name}' (allowed: A-Z a-z 0-9 _ -)")
            }
            Error::ProfileNotFound(name) => write!(f, "profile '{name}' not found"),
            Error::ProfileExists(name) => write!(f, "profile '{name}' already exists"),
            Error::NoDefaultProfile => write!(
                f,
                "no default profile set; pass --profile NAME or run `pushit profile use NAME`"
            ),
            Error::NoConfigHome => write!(f, "could not determine a config directory"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<ron::Error> for Error {
    fn from(e: ron::Error) -> Self {
        Error::Ron(e)
    }
}

impl From<ron::error::SpannedError> for Error {
    fn from(e: ron::error::SpannedError) -> Self {
        Error::RonParse(e)
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}
