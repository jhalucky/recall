use crate::chunker::chunk_document;
use crate::database::Database;
use crate::document::Document;
use crate::embedding::EmbeddingClient;
use crate::error::RecallError;
use crate::metadata::{self, MetadataValue};
use crate::vector::Vector;

pub fn process_document(
    document: &Document,
    chunk_size: usize,
    embedder: &EmbeddingClient,
    database: &mut Database,
) -> Result<usize, RecallError> {
    let chunks = chunk_document(document, chunk_size, 1);

    let mut inserted = 0;

    for chunk in chunks {
        let embedding = embedder.embed(&chunk.text)?;

        let mut metadata = document.metadata.clone();

        metadata.insert(
            "document_id".to_string(),
            MetadataValue::String(document.id.clone()),
        );
        metadata.insert(
            "chunk_index".to_string(),
            MetadataValue::Integer(chunk.chunk_index as i64),
        );

        metadata.insert(
            "text".to_string(),
            MetadataValue::String(chunk.text.clone()),
        );

        let vector = Vector {
            id: chunk.id,
            values: embedding,
            metadata,
        };

        database.upsert(vector);

        inserted += 1;
    }

    Ok(inserted)
}

#[cfg(test)]

mod tests {
    use std::{assert_eq, collections::HashMap, vec};

    use crate::{database, document, vector};

    use super::*;

    #[test]
    fn test_process_document() {
        let document = Document {
            id: String::from("doc_001"),
            text: String::from(
                "Rust is a systems programming language. \
                  Rust provides memory safety without a garbage collector.",
            ),
            metadata: HashMap::new(),
        };

        let embedder = EmbeddingClient::new("http://127.0.0.1:8000".to_string());

        let mut database = Database::new();

        let inserted = process_document(&document, 4, &embedder, &mut database).unwrap();

        assert!(inserted > 0);

        let first_chunk = database.get("doc_001_chunk_0");

        assert!(first_chunk.is_some());

        let vector = first_chunk.unwrap();

        assert_eq!(vector.id, "doc_001_chunk_0");
        assert_eq!(vector.values.len(), 384);

        assert_eq!(
            vector.metadata.get("document_id"),
            Some(&MetadataValue::String("doc_001".to_string()))
        );
        assert_eq!(
            vector.metadata.get("chunk_index"),
            Some(&MetadataValue::Integer(0))
        );

        assert!(vector.metadata.contains_key("text"));
    }

    #[test]
    fn test_document_to_semantic_search() {
        let document = Document {
            id: String::from("rust_doc"),
            text: String::from(
                "Rust provides memory safety through ownership and borrowing. \
             The compiler checks these rules at compile time.",
            ),
            metadata: HashMap::new(),
        };

        let embedder = EmbeddingClient::new("http://127.0.0.1:8000".to_string());

        let mut database = Database::new();

        process_document(&document, 8, &embedder, &mut database).unwrap();

        let query = "How does prevent memory problem?";
        let query_vector = embedder.embed(query).unwrap();

        let results = database.search(&query_vector, 2).unwrap();

        assert!(!results.is_empty());

        assert_eq!(results[0].document_id, "rust_doc");
        assert!(results[0].score > 0.0);
        assert!(!results[0].text.is_empty());
    }
}
