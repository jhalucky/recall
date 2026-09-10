use crate::database::Database;
use crate::embedding::EmbeddingClient;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::search_result::SearchResult;

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub top_k: usize,
    pub document_id: Option<String>,
    pub min_score: Option<f32>,
}

pub struct Retriever<'a> {
    database: &'a Database,
    embedder: &'a EmbeddingClient,
}

impl<'a> Retriever<'a> {
    pub fn new(database: &'a Database, embedder: &'a EmbeddingClient) -> Self {
        Self { database, embedder }
    }

    pub fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, RecallError> {
        let query_vector = self.embedder.embed(query)?;

        let results = match options.document_id {
            Some(document_id) => self.database.search_with_filter(
                &query_vector,
                options.top_k,
                "document_id",
                &MetadataValue::String(document_id),
            )?,
            None => self.database.search(&query_vector, options.top_k)?,
        };

        let results = match options.min_score {
            Some(min_score) => results
                .into_iter()
                .filter(|result| result.score >= min_score)
                .collect(),
            None => results,
        };

        Ok(results)
    }
}
