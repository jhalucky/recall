use crate::database::Database;
use crate::embedding::EmbeddingClient;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::search_result::SearchResult;

pub struct Retriever<'a> {
    database: &'a Database,
    embedder: &'a EmbeddingClient
}

impl<'a> Retriever<'a> {
    pub fn new(
        database: &'a Database,
        embedder: &'a EmbeddingClient
    ) -> Self {
        Self {
            database,
            embedder
        }
    }

    pub fn search(
        &self,
        query: &str,
        top_k: usize
    ) -> Result<Vec<SearchResult>, RecallError> {
        let query_vector = self.embedder.embed(query)?;

        self.database.search(&query_vector, top_k)
    }

    pub fn search_document(
        &self,
        query: &str,
        document_id: &str,
        top_k: usize
    ) -> Result<Vec<SearchResult>, RecallError> {
        let query_vector = self.embedder.embed(query)?;

        self.database.search_with_filter(
            &query_vector,
            top_k,
            "document_id",
            &MetadataValue::String(document_id.to_string())
        )
    }
}