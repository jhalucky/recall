use std::collections::HashMap;

use crate::vector::Vector;
use crate::error::RecallError;
use crate::search_result::SearchResult;
use crate::similarity::cosine_similarity;

pub struct Database {
    vectors: HashMap<String, Vector>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            vectors: HashMap::new(),
        }
    }

    pub fn insert(&mut self, vector: Vector) {
        self.vectors.insert(vector.id.clone(), vector);
    }

    pub fn get(&self, id: &str) -> Option<&Vector> {
        self.vectors.get(id)
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>, RecallError> {
        let mut results = Vec::new();

        for vector in self.vectors.values() {
            let score = cosine_similarity(query, &vector.values)?;

            results.push(SearchResult {
                id: vector.id.clone(),
                score,
            });
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results.truncate(top_k);

        Ok(results)
    }
}



