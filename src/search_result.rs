#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub document_id: String,
    pub chunk_index: usize,
    pub text: String,
}
