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
    use std::{assert_eq, collections::HashMap};

    use crate::{database, document};

    use super::*;

    #[test]
    fn test_process_document() {
        let document = Document {
            id: String::from("doc_001"),
            text: String::from("Rust is a systems programming language used for fast software"),
            metadata: HashMap::new(),
        };

        let embedder = EmbeddingClient::new("http://127.0.0.1:8000".to_string());

        let mut database = Database::new();

        let inserted = process_document(&document, 4, &embedder, &mut database).unwrap();

        assert_eq!(inserted, 3);
    }
}
