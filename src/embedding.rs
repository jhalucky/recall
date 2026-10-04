use serde::{Deserialize, Serialize};

use crate::error::RecallError;

#[derive(Serialize)]
struct EmbedRequest {
    text: String,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, RecallError>;
}
pub struct EmbeddingClient {
    base_url: String,
}

impl EmbeddingClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl EmbeddingProvider for EmbeddingClient {
    fn embed(&self, text: &str) -> Result<Vec<f32>, RecallError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()?;

        let url = format!("{}/embed", self.base_url);
        let request = EmbedRequest {
            text: text.to_string(),
        };

        let mut last_error = None;

        for attempt in 1..=4 {
            match client.post(&url).json(&request).send() {
                Ok(response) => {
                    let response = response.error_for_status()?;
                    let result: EmbedResponse = response.json()?;
                    return Ok(result.embedding);
                }
                Err(error) => {
                    last_error = Some(error);
                    if attempt < 4 {
                        std::thread::sleep(std::time::Duration::from_secs(5 * attempt));
                    }
                }
            }
        }

        Err(last_error.expect("embedding request failed").into())
    }
}

#[cfg(test)]

mod tests {
    use std::assert_eq;

    use crate::embedding;

    use super::*;

    #[test]
    fn test_embedding_client() {
        let client = EmbeddingClient::new("http://127.0.0.1:8001".to_string());

        let embedding = client
            .embed("Rust is a systems programming language")
            .unwrap();

        assert_eq!(embedding.len(), 384);
    }
}
