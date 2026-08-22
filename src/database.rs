use std::collections::HashMap;

use crate::error::RecallError;
use crate::search_result::SearchResult;
use crate::similarity::cosine_similarity;
use crate::vector::Vector;

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
        println!("Before sorting:");

        for result in &results {
            println!("{} -> {}", result.id, result.score);
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        println!("After sorting:");

        for result in &results {
            println!("{} -> {}", result.id, result.score);
        }

        results.truncate(top_k);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use super::*;
    use crate::{database, vector::Vector};

    #[test]
    fn test_insert_and_get_vector() {
        let mut database = Database::new();

        let vector = Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 2.0, 3.0],
        };

        database.insert(vector);
        let result = database.get("doc_001");

        let result = result.unwrap();

        assert_eq!(result.id, "doc_001");
        assert_eq!(result.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_get_missing_vector() {
        let database = Database::new();

        let result = database.get("does not exist");
        assert!(result.is_none());
    }

    #[test]
    fn test_search_returns_top_k_results() {
        let mut database = Database::new();

        database.insert(Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 0.0],
        });

        database.insert(Vector {
            id: String::from("doc_002"),
            values: vec![0.0, 1.0],
        });

        database.insert(Vector {
            id: String::from("doc_003"),
            values: vec![0.8, 0.2],
        });

        let query = vec![1.0, 0.0];

        let results = database.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].id, "doc_001");
        assert_eq!(results[1].id, "doc_003");
    }

    #[test]
    fn test_seacrh_top_k_larger_than_database() {
        let mut database = Database::new();

        database.insert(Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 0.0],
        });

        database.insert(Vector {
            id: String::from("doc_002"),
            values: vec![0.0, 1.0],
        });

        let query = vec![1.0, 0.0];

        let results = database.search(&query, 10).unwrap();
        assert_eq!(results.len(), 2)
    }

    #[test]
    fn test_search_with_zero_top_k() {
        let mut database = Database::new();

        database.insert(Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 0.0],
        });

        database.insert(Vector {
            id: String::from("doc_002"),
            values: vec![0.0, 1.0],
        });

        let query = vec![1.0, 0.0];
        let results = database.search(&query, 0).unwrap();

        assert!(results.is_empty());
    }
}
