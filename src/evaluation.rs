use serde::Deserialize;
use std::fs;

use crate::database::Database;
use crate::embedding::EmbeddingClient;
use crate::error::RecallError;

#[derive(Debug, Deserialize)]
pub struct EvaluationQuery {
    pub query: String,
    pub expected_document: String,
}

pub fn load_queries(path: &str) -> Result<Vec<EvaluationQuery>, RecallError> {
    let content = fs::read_to_string(path)?;
    let queries: Vec<EvaluationQuery> = serde_json::from_str(&content)?;

    Ok(queries)
}

pub fn evaluate(
    database: &Database,
    embedder: &EmbeddingClient,
    queries: &[EvaluationQuery],
    top_k: usize,
) -> Result<(usize, usize), RecallError> {
    let mut top_1_correct = 0;
    let mut top_k_correct = 0;

    for evaluation_query in queries {
        let query_vector = embedder.embed(&evaluation_query.query)?;

        let results = database.search(&query_vector, top_k)?;

        if let Some(first) = results.first() {
            if first.document_id == evaluation_query.expected_document {
                top_1_correct += 1;
            }
        }

        if results
            .iter()
            .any(|result| result.document_id == evaluation_query.expected_document)
        {
            top_k_correct += 1;
        }
    }

    Ok((top_1_correct, top_k_correct))
}
