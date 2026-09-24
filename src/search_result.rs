use serde::Serialize;

use crate::metadata::MetadataValue;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResult {
    pub document_id: String,
    pub chunk_id: String,
    pub chunk_index: usize,
    pub text: String,
    pub score: f32,
    pub metadata: HashMap<String, MetadataValue>,
}
