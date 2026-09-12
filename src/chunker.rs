use std::collections::HashMap;

use crate::{document::Document, metadata::MetadataValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub text: String,
    pub chunk_index: usize,
    pub metadata: HashMap<String, MetadataValue>,
}

pub fn chunk_document(document: &Document, chunk_size: usize, overlap: usize) -> Vec<Chunk> {
    if chunk_size == 0 || overlap >= chunk_size {
        return Vec::new();
    }

    let words: Vec<&str> = document.text.split_whitespace().collect();

    let mut chunks = Vec::new();
    let step = chunk_size - overlap;
    let mut start = 0;
    let mut chunk_index = 0;

    while start < words.len() {
        let end = usize::min(start + chunk_size, words.len());
        let chunk_words = &words[start..end];

        let text = chunk_words.join(" ");

        chunks.push(Chunk {
            id: format!("{}_chunk_{}", document.id, chunk_index),
            document_id: document.id.clone(),
            text,
            chunk_index,
            metadata: document.metadata.clone(),
        });

        chunk_index += 1;
        start += step;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        document::{self, Document},
        metadata,
    };
    use std::{assert_eq, collections::HashMap};

    #[test]
    fn test_chunk_document() {
        let document = Document {
            id: String::from("doc_001"),
            text: String::from("Rust is a systems programming language used for fast software"),
            metadata: HashMap::new(),
        };

        let chunks = chunk_document(&document, 4, 1);

        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].text, "Rust is a systems");
        assert_eq!(chunks[1].text, "systems programming language used");
        assert_eq!(chunks[2].text, "used for fast software");
        assert_eq!(chunks[3].text, "software");

        assert_eq!(chunks[0].document_id, "doc_001");
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert_eq!(chunks[2].chunk_index, 2);
        assert_eq!(chunks[3].chunk_index, 3);
    }

    #[test]
    fn test_chunk_presence_document_metadata() {
        let mut metadata = HashMap::new();

        metadata.insert(
            "title".to_string(),
            crate::metadata::MetadataValue::String("Rust Notes".to_string()),
        );

        metadata.insert(
            "page".to_string(),
            crate::metadata::MetadataValue::Integer(10),
        );

        let document = Document {
            id: "doc_001".to_string(),
            text: "Rust is fast and safe".to_string(),
            metadata,
        };

        let chunks = chunk_document(&document, 4, 1);

        assert_eq!(
            chunks[0].metadata.get("title"),
            Some(&crate::metadata::MetadataValue::String(
                "Rust Notes".to_string()
            ))
        );

        assert_eq!(
            chunks[0].metadata.get("page"),
            Some(&crate::metadata::MetadataValue::Integer(10))
        );
    }

    #[test]
    fn test_chunk_ids_are_stable_and_unique() {
        let document = Document {
            id: "doc_001".to_string(),
            text: "one two three four five six seven eight".to_string(),
            metadata: HashMap::new(),
        };

        let chunks_first = chunk_document(&document, 4, 1);
        let chunks_second = chunk_document(&document, 4, 1);

        // Same document + same chunking configuration
        // must produce the same chunk IDs.
        for (first, second) in chunks_first.iter().zip(chunks_second.iter()) {
            assert_eq!(first.id, second.id);
        }

        // Different chunks must have different IDs.
        assert_ne!(chunks_first[0].id, chunks_first[1].id);
        assert_ne!(chunks_first[1].id, chunks_first[2].id);

        // IDs should contain the document identity.
        assert_eq!(chunks_first[0].id, "doc_001_chunk_0");
        assert_eq!(chunks_first[1].id, "doc_001_chunk_1");
    }
}
