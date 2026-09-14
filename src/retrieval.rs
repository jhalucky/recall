use crate::database::Database;
use crate::embedding::EmbeddingProvider;
use crate::error::RecallError;
use crate::filter::MetadataFilter;
use crate::metadata::MetadataValue;
use crate::search_result::SearchResult;
use crate::diagnostics::RetrievalDiagnostics;

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub top_k: usize,
    pub document_id: Option<String>,
    pub min_score: Option<f32>,
    pub filters: Vec<MetadataFilter>,
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

        let mut filters = options.filters.clone();

        if let Some(document_id) = options.document_id.as_ref() {
            filters.push(MetadataFilter::new(
                "document_id".to_string(),
                MetadataValue::String(document_id.clone()),
            ));
        }

        let results = if filters.is_empty() {
            self.database.search(&query_vector, options.top_k)?
        } else {
            self.database
                .search_with_filters(&query_vector, options.top_k, &filters)?
        };

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

    pub fn search_with_diagnostics(
        &self,
        query: &str,
        options: SearchOptions
    ) -> Result<(Vec<SearchResult>, RetrievalDiagnostics), RecallError> {
        let query_vector = self.embedder.embed(query)?;

        let mut filters = options.filters.clone();

        if let Some(document_id) = options.document_id.as_ref() {
            filters.push(MetadataFilter::new(
                "document_id".to_string(),
                MetadataValue::String(document_id.clone())
            ));
        }

        let candidates = if filters.is_empty() {
            self.database.search(&query_vector, options.top_k)?
        } else {
            self.database
                .search_with_filters(&query_vector, options.top_k, &filters)?
        };

        let candidates_count = candidates.len();

        let results_before_min_score = candidates.len();

        let results = if let Some(min_score) = options.min_score {
            candidates
                .into_iter()
                .filter(|result| result.score >= min_score)
                .collect()
        } else {
            candidates
        };

        let results_after_min_score = results.len();

        let diagnostics = RetrievalDiagnostics {
            candidates: candidates_count,
            results_before_min_score,
            results_after_min_score
        };

        Ok((results, diagnostics))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{metadata, vector::Vector};
    use std::{assert_eq, collections::HashMap, vec};

    struct TestEmbedder;

    impl EmbeddingProvider for TestEmbedder {
        fn embed(&self, _text: &str) -> Result<Vec<f32>, RecallError> {
            Ok(vec![1.0, 0.0, 0.0])
        }
    }

    fn test_embedding_config() -> crate::EmbeddingConfig {
        crate::EmbeddingConfig {
            provider: "test".to_string(),
            model: "test-model".to_string(),
            dimension: 3,
            version: "1".to_string(),
        }
    }

    #[test]
    fn test_retriever_with_multiple_filters() {
        let mut database = Database::new(test_embedding_config());

        let mut metadata_1 = HashMap::new();
        metadata_1.insert(
            "subject".to_string(),
            MetadataValue::String("DBMS".to_string()),
        );
        metadata_1.insert("semester".to_string(), MetadataValue::Integer(5));

        database
            .insert(Vector {
                id: "chunk_1".to_string(),
                values: vec![1.0, 0.0, 0.0],
                metadata: metadata_1,
            })
            .unwrap();

        let mut metadata_2 = HashMap::new();
        metadata_2.insert(
            "subject".to_string(),
            MetadataValue::String("OS".to_string()),
        );
        metadata_2.insert("semester".to_string(), MetadataValue::Integer(5));

        database
            .insert(Vector {
                id: "chunk_2".to_string(),
                values: vec![0.9, 0.1, 0.0],
                metadata: metadata_2,
            })
            .unwrap();

        let embedder = TestEmbedder;
        let retriever = Retriever::new(&database, &embedder);

        let options = SearchOptions {
            top_k: 10,
            document_id: None,
            min_score: None,
            filters: vec![
                MetadataFilter::new(
                    "subject".to_string(),
                    MetadataValue::String("DBMS".to_string()),
                ),
                MetadataFilter::new("semester".to_string(), MetadataValue::Integer(5)),
            ],
        };

        let results = retriever.search("database systems", options).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "chunk_1");
    }

    #[test]
    fn test_retriever_combines_document_id_and_metadata_filters() {
        let mut database = Database::new(test_embedding_config());

        let mut metadata_1 = HashMap::new();
        metadata_1.insert(
            "document_id".to_string(),
            MetadataValue::String("doc_1".to_string()),
        );
        metadata_1.insert(
            "subject".to_string(),
            MetadataValue::String("DBMS".to_string()),
        );

        database
            .insert(Vector {
                id: "chunk_1".to_string(),
                values: vec![1.0, 0.0, 0.0],
                metadata: metadata_1,
            })
            .unwrap();

        let mut metadata_2 = HashMap::new();
        metadata_2.insert(
            "document_id".to_string(),
            MetadataValue::String("doc_2".to_string()),
        );
        metadata_2.insert(
            "subject".to_string(),
            MetadataValue::String("DBMS".to_string()),
        );

        database
            .insert(Vector {
                id: "chunk_2".to_string(),
                values: vec![0.9, 0.1, 0.0],
                metadata: metadata_2,
            })
            .unwrap();

        let embedder = TestEmbedder;
        let retriever = Retriever::new(&database, &embedder);

        let options = SearchOptions {
            top_k: 10,
            document_id: Some("doc_1".to_string()),
            min_score: None,
            filters: vec![MetadataFilter::new(
                "subject".to_string(),
                MetadataValue::String("DBMS".to_string()),
            )],
        };

        let results = retriever.search("database systems", options).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "chunk_1");
    }

    #[test]
    fn test_search_With_diagnostics() {
        let mut database = Database::new(test_embedding_config());

        let mut metadata = HashMap::new();
        metadata.insert(
            "subject".to_string(),
            MetadataValue::String("DBMS".to_string())
        );

        database
            .insert(Vector {
                id: "chunk_1".to_string(),
                values: vec![1.0, 0.0, 0.0],
                metadata
            })
            .unwrap();

        let embedder = TestEmbedder;
        let retriever = Retriever::new(&database, &embedder);

        let options = SearchOptions {
            top_k: 10,
            document_id: None,
            min_score: None,
            filters: vec![]
        };

        let (results, diagnostics) = retriever
            .search_with_diagnostics("database systems", options)
            .unwrap();


        assert_eq!(results.len(), 1);
        assert_eq!(diagnostics.candidates, 1);
        assert_eq!(diagnostics.results_before_min_score, 1);
        assert_eq!(diagnostics.results_after_min_score, 1);
    }
}
