use serde::Deserialize;
use std::fs;

use crate::database::Database;
use crate::embedding::EmbeddingClient;
use crate::error::RecallError;

#[derive(Debug, Deserialize)]
pub struct EvaluationQuery {
    pub query: String,
    pub relevant_chunks: Vec<String>,
}

pub fn load_queries(path: &str) -> Result<Vec<EvaluationQuery>, RecallError> {
    let content = fs::read_to_string(path)?;
    let queries: Vec<EvaluationQuery> = serde_json::from_str(&content)?;

    Ok(queries)
}

pub fn evaluate_detailed(
    database: &Database,
    embedder: &EmbeddingClient,
    queries: &[EvaluationQuery],
    top_k: usize,
) -> Result<(usize, usize, f32), RecallError> {
    let mut top_1_correct = 0;
    let mut top_k_correct = 0;
    let mut reciprocal_rank_sum = 0.0;

    for (index, evaluation_query) in queries.iter().enumerate() {
        let query_vector = embedder.embed(&evaluation_query.query)?;

        let results = database.search(&query_vector, top_k)?;

        let top_1_match = results
            .first()
            .map(|result| evaluation_query.relevant_chunks.contains(&result.id))
            .unwrap_or(false);

        let first_relevant_rank = results
            .iter()
            .position(|result| evaluation_query.relevant_chunks.contains(&result.id));

        let top_k_match = first_relevant_rank.is_some();

        if top_1_match {
            top_1_correct += 1;
        }

        if top_k_match {
            top_k_correct += 1;
        }

        if let Some(rank) = first_relevant_rank {
            reciprocal_rank_sum += 1.0 / (rank as f32 + 1.0);
        }

        println!();
        println!("Query {}/{}", index + 1, queries.len());
        println!("Query: {}", evaluation_query.query);
        println!("Relevant chunks: {:?}", evaluation_query.relevant_chunks);
        println!();

        for (rank, result) in results.iter().enumerate() {
            let marker = if evaluation_query.relevant_chunks.contains(&result.id) {
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
            println!("Result: ~ Relevant chunk found in Top-{}", top_k);
        } else {
            println!("Result: ✗ Relevant chunk not found");
        }
    }

    let mrr = if queries.is_empty() {
        0.0
    } else {
        reciprocal_rank_sum / queries.len() as f32
    };

    Ok((top_1_correct, top_k_correct, mrr))
}
