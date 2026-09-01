use crate::document::Document;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub text: String,
    pub chunk_index: usize,
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
        });

        chunk_index += 1;
        start += step;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use std::collections::HashMap;

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
}
