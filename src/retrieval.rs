use crate::database::Database;
use crate::embedding::EmbeddingProvider;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::search_result::SearchResult;
use crate::filter::MetadataFilter;

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub top_k: usize,
    pub document_id: Option<String>,
    pub min_score: Option<f32>,
}

pub struct Retriever<'a> {
    database: &'a Database,
    embedder: &'a dyn EmbeddingProvider,
}

impl<'a> Retriever<'a> {
    pub fn new(database: &'a Database, embedder: &'a dyn EmbeddingProvider) -> Self {
        Self { database, embedder }
    }

    pub fn search(
    &self,
    query: &str,
    options: SearchOptions,
) -> Result<Vec<SearchResult>, RecallError> {
    let query_vector = self.embedder.embed(query)?;

    let results = match options.document_id.as_ref() {
        Some(document_id) => {
            let filter = MetadataFilter::new(
                "document_id".to_string(),
                MetadataValue::String(document_id.clone()),
            );

            self.database.search_with_filter(
                &query_vector,
                options.top_k,
                &filter,
            )
        }
        None => self.database.search(&query_vector, options.top_k),
    }?;

    let results = if let Some(min_score) = options.min_score {
        results
            .into_iter()
            .filter(|result| result.score >= min_score)
            .collect()
    } else {
        results
    };

    Ok(results)
}
}

