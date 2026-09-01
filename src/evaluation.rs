use serde::Deserialize;
use std::{fs, print, println};

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

pub fn evaluate_detailed(
    database: &Database,
    embedder: &EmbeddingClient,
    queries: &[EvaluationQuery],
    top_k: usize,
) -> Result<(usize, usize), RecallError> {
    let mut top_1_correct = 0;
    let mut top_k_correct = 0;

    for (index, evaluation_query) in queries.iter().enumerate() {
        let query_vector = embedder.embed(&evaluation_query.query)?;

        let results = database.search(&query_vector, top_k)?;

        let top_1_match = results
            .first()
            .map(|result| result.document_id == evaluation_query.expected_document)
            .unwrap_or(false);

        let top_k_match = results
            .iter()
            .any(|result| result.document_id == evaluation_query.expected_document);

        if top_1_match {
            top_1_correct += 1;
        }

        if top_k_match {
            top_k_correct += 1;
        }

        println!();
        println!("Query {}/{}", index + 1, queries.len());
        println!("Query: {}", evaluation_query.query);
        println!("Expected document: {}", evaluation_query.expected_document);
        println!();

        for (rank, result) in results.iter().enumerate() {
            let marker = if result.document_id == evaluation_query.expected_document {
                "✓"
            } else {
                " "
            };

            println!(
                "{}. {} {} -> {:.6}",
                rank + 1,
                marker,
                result.id,
                result.score
            );

            println!("  {}", result.text);
        }

        if top_1_match {
            println!("Result: ✓ Top-1 correct");
        } else if top_k_match {
            println!("Result: ~ Found in Top-{}", top_k);
        } else {
            println!("Result: ✗ Not found in Top-{}", top_k);
        }
    }

    Ok((top_1_correct, top_k_correct))
}
