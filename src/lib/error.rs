use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Error {
    bb8(bb8::RunError<tokio_postgres::Error>),
    Io(std::io::Error),
    Json(serde_json::Error),
    Postgres(tokio_postgres::Error),
    Regex(regex::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::bb8(ref err) => write!(f, "{err}"),
            Self::Io(ref err) => write!(f, "{err}"),
            Self::Json(ref err) => write!(f, "{err}"),
            Self::Postgres(ref err) => write!(f, "{err}"),
            Self::Regex(ref err) => write!(f, "{err}"),
        }
    }
}

impl From<bb8::RunError<tokio_postgres::Error>> for Error {
    fn from(err: bb8::RunError<tokio_postgres::Error>) -> Self {
        Self::bb8(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::Postgres(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Self::Regex(err)
    }
}
