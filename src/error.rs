use std::fmt;

#[derive(Debug)]
pub enum RecallError {
    DimensionMismatch { query: usize, stored: usize },

    VectorAlreadyExists,

    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    ReqwestError(reqwest::Error),
}

impl fmt::Display for RecallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecallError::DimensionMismatch { query, stored } => {
                write!(
                    f,
                    "embedding dimension mismatch: query={}, stored={}",
                    query, stored
                )
            }

            RecallError::VectorAlreadyExists => {
                write!(f, "vector already exists")
            }

            RecallError::IoError(error) => {
                write!(f, "I/O error: {}", error)
            }

            RecallError::SerializationError(error) => {
                write!(f, "serialization error: {}", error)
            }

            RecallError::ReqwestError(error) => {
                write!(f, "embedding request error: {}", error)
            }
        }
    }
}

impl std::error::Error for RecallError {}

impl From<std::io::Error> for RecallError {
    fn from(error: std::io::Error) -> Self {
        RecallError::IoError(error)
    }
}

impl From<serde_json::Error> for RecallError {
    fn from(error: serde_json::Error) -> Self {
        RecallError::SerializationError(error)
    }
}

impl From<reqwest::Error> for RecallError {
    fn from(error: reqwest::Error) -> Self {
        RecallError::ReqwestError(error)
    }
}
