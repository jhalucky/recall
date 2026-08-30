use crate::document::Document;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub text: String,
    pub chunk_index: usize,
}

pub fn chunk_document(document: &Document, chunk_size: usize) -> Vec<Chunk> {
    let words: Vec<&str> = document.text.split_whitespace().collect();

    let mut chunks = Vec::new();

    for (chunk_index, chunk_words) in words.chunks(chunk_size).enumerate() {
        let text = chunk_words.join(" ");

        chunks.push(Chunk {
            id: format!("{}_chunk_{}", document.id, chunk_index),
            document_id: document.id.clone(),
            text,
            chunk_index,
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use crate::document;

    use super::*;
    use std::{assert_eq, collections::HashMap};

    #[test]
    fn test_chunk_document() {
        let document = Document {
            id: String::from("doc_001"),
            text: String::from("Rust is a systems programming language used for fast software"),
            metadata: HashMap::new(),
        };

        let chunks = chunk_document(&document, 4);

        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].text, "Rust is a systems");
        assert_eq!(chunks[1].text, "programming language used for");
        assert_eq!(chunks[2].text, "fast software");

        assert_eq!(chunks[0].document_id, "doc_001");
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert_eq!(chunks[2].chunk_index, 2);
    }
}
