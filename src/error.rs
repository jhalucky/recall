#[derive(Debug)]
pub enum RecallError {
    DimensionMismatch { query: usize, stored: usize },

    VectorAlreadyExists,

    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    ReqwestError(reqwest::Error),
}

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
