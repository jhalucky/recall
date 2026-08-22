mod vector;
mod search_result;

use crate::vector::Vector;
use crate::search_result::SearchResult;
use std::{collections::HashMap, println};


struct Database {
    vectors: HashMap<String, Vector>,
}

enum RecallError{
    DimensionMismatch{
        query: usize,
        stored: usize
    }
}
impl Database {
    fn new() -> Database {
        Database {
            vectors: HashMap::new(),
        }
    }

    fn insert(&mut self, vector: Vector) {
        self.vectors.insert(vector.id.clone(), vector);
    }

    fn get(&self, id: &str) -> Option<&Vector> {
        self.vectors.get(id)
    }

    fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>, RecallError> {
        let mut results = Vec::new();

        for vector in self.vectors.values() {
            let score = cosine_similarity(query, &vector.values)?;

            results.push(SearchResult {
                id: vector.id.clone(),
                score
            });
        }
        results.sort_by(|a,b| {
            b.score.partial_cmp(&a.score).unwrap()
        });

        results.truncate(top_k);

        Ok(results)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, RecallError> {
    if a.len() != b.len() {
        return Err(RecallError::DimensionMismatch {
            query: a.len(), 
            stored: b.len() 
        });
    } 
    let mut dot_product = 0.0;
    let mut magnitude_a = 0.0;
    let mut magnitude_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];

        magnitude_a += a[i] * a[i];
        magnitude_b += b[i] * b[i];
    }

    Ok(
        dot_product 
            / (magnitude_a.sqrt() * magnitude_b.sqrt())
    )
}
fn main() {
    let mut database = Database::new();

    let vector = Vector {
        id: String::from("doc_001"),
        values: vec![0.12, 0.55, 0.81],
    };

    let vector2 = Vector {
        id: String::from("doc_002"),
        values: vec![0.91, 0.12, 0.44],
    };

    let vector3 = Vector {
        id: String::from("doc_003"),
        values: vec![0.33, 0.72, 0.48],
    };

    database.insert(vector);
    database.insert(vector2);
    database.insert(vector3);

    let query = vec![0.10, 0.51, 0.68];
    match database.search(&query, 2) {
    Ok(results) => {
        for result in results {
            println!("{} → {}", result.id, result.score);
        }
    }

    Err(RecallError::DimensionMismatch { query, stored }) => {
        println!(
            "Search failed: query has {} dimensions, stored vector has {} dimensions",
            query,
            stored
        );
    }
}

}