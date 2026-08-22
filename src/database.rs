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


#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::vector::Vector;

    #[test]
    fn test_insert_and_get_vector() {
        let mut database = Database::new();

        let vector = Vector {
            id: String::from("doc_001"),
            values: vec![1.0,2.0,3.0]
        };

        database.insert(vector);
        let result = database.get("doc_001");

        assert!(result.is_some());
    }
}
