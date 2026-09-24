use crate::config::EmbeddingConfig;
use crate::database::Database;
use crate::document::Document;
use crate::embedding::{EmbeddingClient, EmbeddingProvider};
use crate::error::RecallError;
use crate::pipeline::process_document;
use crate::retrieval::{Retriever, SearchOptions};
use crate::search_result::SearchResult;

pub struct RecallEngine {
    database: Database,
    embedder: Box<dyn EmbeddingProvider>,
}

impl RecallEngine {
    pub fn new(embedding_url: String, embedding_config: EmbeddingConfig) -> Self {
        Self {
            database: Database::new(embedding_config),
            embedder: Box::new(EmbeddingClient::new(embedding_url)),
        }
    }

    pub fn add_document(
        &mut self,
        document: &Document,
        chunk_size: usize,
    ) -> Result<usize, RecallError> {
        process_document(
            document,
            chunk_size,
            self.embedder.as_ref(),
            &mut self.database,
        )
    }

    pub fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, RecallError> {
        let retriever = Retriever::new(&self.database, self.embedder.as_ref());

        retriever.search(query, options)
    }

    pub fn delete_document(&mut self, document_id: &str) -> usize {
        self.database.delete_by_metadata(
            "document_id",
            &crate::metadata::MetadataValue::String(document_id.to_string()),
        )
    }

    pub fn list_documents(&self) -> Vec<(String, usize)> {
        self.database.list_documents()
    }

    pub fn save(&self, path: &str) -> Result<(), RecallError> {
        self.database.save(path)
    }

    pub fn load(path: &str, embedding_url: String) -> Result<Self, RecallError> {
        Ok(Self {
            database: Database::load(path)?,
            embedder: Box::new(EmbeddingClient::new(embedding_url)),
        })
    }

    pub fn add_document_pages(
        &mut self,
        document_id: &str,
        pages: &[crate::document::DocumentPage],
        metadata: &std::collections::HashMap<String, crate::metadata::MetadataValue>,
        chunk_size: usize,
    ) -> Result<usize, crate::error::RecallError> {
        crate::pipeline::process_document_pages(
            document_id,
            pages,
            metadata,
            chunk_size,
            self.embedder.as_ref(),
            &mut self.database,
        )
    }
}
