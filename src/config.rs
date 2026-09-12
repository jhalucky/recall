#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub dimension: usize,
    pub version: String
}

