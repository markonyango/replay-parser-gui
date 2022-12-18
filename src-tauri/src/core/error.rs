use std::{string::FromUtf8Error, convert::Infallible};

use color_eyre::Report;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[allow(clippy::enum_variant_names)]
pub enum ParserAppError {
    #[error("Parser lib error: {0}")]
    ParserLibError(String),
    #[error("Could not find logfile")]
    LogfileNotFoundError,
    #[error("Error while parsing logfile: {0}")]
    LogfileParseError(String),
    #[error("Could not find replay file")]
    ReplayNotFoundError,
    #[error("Generic error: {0}")]
    GenericError(String),
}

pub type ParserAppResult<T, E = ParserAppError> = color_eyre::Result<T, E>;

impl Serialize for ParserAppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<std::io::Error> for ParserAppError {
    fn from(e: std::io::Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<FromUtf8Error> for ParserAppError {
    fn from(e: FromUtf8Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<notify::Error> for ParserAppError {
    fn from(e: notify::Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<tauri::Error> for ParserAppError {
    fn from(e: tauri::Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<serde_json::Error> for ParserAppError {
    fn from(e: serde_json::Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<reqwest::Error> for ParserAppError {
    fn from(e: reqwest::Error) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<Vec<notify::Error>> for ParserAppError {
    fn from(_: Vec<notify::Error>) -> Self {
        ParserAppError::GenericError("Notify error".into())
    }
}

impl From<Infallible> for ParserAppError {
    fn from(e: Infallible) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}

impl From<Report> for ParserAppError {
    fn from(e: Report) -> Self {
        ParserAppError::GenericError(e.to_string())
    }
}
