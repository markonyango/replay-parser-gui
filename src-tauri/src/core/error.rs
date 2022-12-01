use std::string::FromUtf8Error;

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
