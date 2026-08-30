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

pub struct EmbeddingClient {
    base_url: String,
}

impl EmbeddingClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, RecallError> {
        let client = reqwest::blocking::Client::new();

        let request = EmbedRequest {
            text: text.to_string(),
        };

        let response = client
            .post(format!("{}/embed", self.base_url))
            .json(&request)
            .send()?;

        let response = response.error_for_status()?;

        let result: EmbedResponse = response.json()?;

        Ok(result.embedding)
    }
}

#[cfg(test)]

mod tests {
    use std::assert_eq;

    use crate::embedding;

    use super::*;

    #[test]
    fn test_embedding_client() {
        let client = EmbeddingClient::new("http://127.0.0.1:8000".to_string());

        let embedding = client
            .embed("Rust is a systems programming language")
            .unwrap();

        assert_eq!(embedding.len(), 384);
    }
}
