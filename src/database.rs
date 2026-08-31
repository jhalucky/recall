use std::collections::HashMap;
use std::fs::File;

use crate::document;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::search_result::SearchResult;
use crate::similarity::cosine_similarity;
use crate::vector::{self, Vector};

pub struct Database {
    vectors: HashMap<String, Vector>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            vectors: HashMap::new(),
        }
    }

    pub fn insert(&mut self, vector: Vector) -> Result<(), RecallError> {
        if self.vectors.contains_key(&vector.id) {
            return Err(RecallError::VectorAlreadyExists);
        }
        self.vectors.insert(vector.id.clone(), vector);

        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Vector> {
        self.vectors.get(id)
    }

    pub fn delete(&mut self, id: &str) -> Option<Vector> {
        self.vectors.remove(id)
    }

    pub fn delete_by_metadata(&mut self, key: &str, value: &MetadataValue) -> usize {
        let ids: Vec<String> = self
            .vectors
            .iter()
            .filter(|(_, vector)| vector.metadata.get(key) == Some(value))
            .map(|(id, _)| id.clone())
            .collect();

        let deleted = ids.len();

        for id in ids {
            self.vectors.remove(&id);
        }

        deleted
    }

    pub fn list_documents(&self) -> Vec<(String, usize)> {
        let mut documents: HashMap<String, usize> = HashMap::new();

        for vector in self.vectors.values() {
            if let Some(MetadataValue::String(document_id)) = vector.metadata.get("document_id") {
                *documents.entry(document_id.clone()).or_insert(0) += 1;
            }
        }

        let mut result: Vec<(String, usize)> = documents.into_iter().collect();

        result.sort_by(|a, b| a.0.cmp(&b.0));

        result
    }

    pub fn upsert(&mut self, vector: Vector) {
        self.vectors.insert(vector.id.clone(), vector);
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>, RecallError> {
        let mut results = Vec::new();

        for vector in self.vectors.values() {
            let score = cosine_similarity(query, &vector.values)?;

            results.push(SearchResult {
                id: vector.id.clone(),
                score,
                document_id: match vector.metadata.get("document_id") {
                    Some(MetadataValue::String(id)) => id.clone(),
                    _ => String::new(),
                },

                chunk_index: match vector.metadata.get("chunk_index") {
                    Some(MetadataValue::Integer(index)) => *index as usize,
                    _ => 0,
                },

                text: match vector.metadata.get("text") {
                    Some(MetadataValue::String(text)) => text.clone(),
                    _ => String::new(),
                },
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results.truncate(top_k);

        Ok(results)
    }

    pub fn save(&self, path: &str) -> Result<(), RecallError> {
        let file = File::create(path)?;

        serde_json::to_writer_pretty(file, &self.vectors)?;

        Ok(())
    }

    pub fn load(path: &str) -> Result<Database, RecallError> {
        let file = File::open(path)?;

        let vectors = serde_json::from_reader(file)?;

        Ok(Database { vectors })
    }

    pub fn search_with_filter(
        &self,
        query: &[f32],
        top_k: usize,
        key: &str,
        value: &MetadataValue,
    ) -> Result<Vec<SearchResult>, RecallError> {
        let mut results = Vec::new();

        for vector in self.vectors.values() {
            if vector.metadata.get(key) != Some(value) {
                continue;
            }

            let score = cosine_similarity(query, &vector.values)?;

            results.push(SearchResult {
                id: vector.id.clone(),
                score,
                document_id: match vector.metadata.get("document_id") {
                    Some(MetadataValue::String(id)) => id.clone(),
                    _ => String::new(),
                },

                chunk_index: match vector.metadata.get("chunk_index") {
                    Some(MetadataValue::Integer(index)) => *index as usize,
                    _ => 0,
                },

                text: match vector.metadata.get("text") {
                    Some(MetadataValue::String(text)) => text.clone(),
                    _ => String::new(),
                },
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results.truncate(top_k);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, fs::metadata, print, vec};

    use serde_json::error::Category::Data;

    use super::*;
    use crate::{database, vector::Vector};

    #[test]
    fn test_insert_and_get_vector() {
        let mut database = Database::new();

        let vector = Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        database.insert(vector).unwrap();
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

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        database
            .insert(Vector {
                id: String::from("doc_002"),
                values: vec![0.0, 1.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        database
            .insert(Vector {
                id: String::from("doc_003"),
                values: vec![0.8, 0.2],
                metadata: HashMap::new(),
            })
            .unwrap();

        let query = vec![1.0, 0.0];

        let results = database.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].id, "doc_001");
        assert_eq!(results[1].id, "doc_003");
    }

    #[test]
    fn test_seacrh_top_k_larger_than_database() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        database
            .insert(Vector {
                id: String::from("doc_002"),
                values: vec![0.0, 1.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let query = vec![1.0, 0.0];

        let results = database.search(&query, 10).unwrap();
        assert_eq!(results.len(), 2)
    }

    #[test]
    fn test_search_with_zero_top_k() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        database
            .insert(Vector {
                id: String::from("doc_002"),
                values: vec![0.0, 1.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let query = vec![1.0, 0.0];
        let results = database.search(&query, 0).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_delete_existing_vector() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let deleted = database.delete("doc_001");

        assert!(deleted.is_some());

        let result = database.get("doc_001");

        assert!(result.is_none());
    }

    #[test]
    fn test_delete_missing_vector() {
        let mut database = Database::new();

        let deleted = database.delete("does_not_exist");

        assert!(deleted.is_none());
    }

    #[test]
    fn test_vector_database() {
        let mut database = Database::new();

        let mut metadata = HashMap::new();

        metadata.insert(
            String::from("title"),
            MetadataValue::String(String::from("Learning Rust")),
        );

        metadata.insert(String::from("year"), MetadataValue::Integer(2026));

        let vector = Vector {
            id: String::from("doc_001"),
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        database.insert(vector).unwrap();

        let result = database.get("doc_001").unwrap();

        assert_eq!(
            result.metadata.get("title"),
            Some(&MetadataValue::String(String::from("Learning Rust")))
        );

        assert_eq!(
            result.metadata.get("year"),
            Some(&MetadataValue::Integer(2026))
        );
    }

    #[test]
    fn test_search_with_filter() {
        let mut database = Database::new();

        let mut rust_metadata = HashMap::new();

        rust_metadata.insert(
            String::from("category"),
            MetadataValue::String(String::from("programming")),
        );

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: rust_metadata,
            })
            .unwrap();

        let mut cooking_metadata = HashMap::new();

        cooking_metadata.insert(
            String::from("category"),
            MetadataValue::String(String::from("cooking")),
        );

        database
            .insert(Vector {
                id: String::from("doc_002"),
                values: vec![1.0, 0.0],
                metadata: cooking_metadata,
            })
            .unwrap();

        let mut python_metadata = HashMap::new();

        python_metadata.insert(
            String::from("category"),
            MetadataValue::String(String::from("programming")),
        );

        database
            .insert(Vector {
                id: String::from("doc_003"),
                values: vec![0.8, 0.2],
                metadata: python_metadata,
            })
            .unwrap();

        let query = vec![1.0, 0.0];

        let filter_value = MetadataValue::String(String::from("programming"));

        let results = database
            .search_with_filter(&query, 10, "category", &filter_value)
            .unwrap();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].id, "doc_001");
        assert_eq!(results[1].id, "doc_003");
    }

    #[test]
    fn test_insert_rejects_duplicate_vector() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let result = database.insert(Vector {
            id: String::from("doc_001"),
            values: vec![0.5, 0.5],
            metadata: HashMap::new(),
        });

        assert!(matches!(result, Err(RecallError::VectorAlreadyExists)));

        let stored = database.get("doc_001").unwrap();

        assert_eq!(stored.values, vec![1.0, 0.0]);
    }

    #[test]
    fn test_upsert_replaces_existing_vector() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 0.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        database.upsert(Vector {
            id: String::from("doc_001"),
            values: vec![0.5, 0.5],
            metadata: HashMap::new(),
        });

        let stored = database.get("doc_001").unwrap();

        assert_eq!(stored.values, vec![0.5, 0.5]);
    }

    #[test]
    fn test_save_database() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let path = "test_recall.json";

        database.save(path).unwrap();

        assert!(std::path::Path::new(path).exists());
        // std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_save_and_load_database() {
        let mut database = Database::new();

        database
            .insert(Vector {
                id: String::from("doc_001"),
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            })
            .unwrap();

        let path = "test_recall.json";

        database.save(path).unwrap();

        let loaded_db = Database::load(path).unwrap();
        let result = loaded_db.get("doc_001").unwrap();

        assert_eq!(result.id, "doc_001");
        assert_eq!(result.values, vec![1.0, 2.0, 3.0]);

        // std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_delete_by_metadata() {
        let mut database = Database::new();

        let mut metadata_1 = HashMap::new();
        metadata_1.insert(
            "document_id".to_string(),
            MetadataValue::String("rust".to_string()),
        );

        let mut metadata_2 = HashMap::new();
        metadata_2.insert(
            "document_id".to_string(),
            MetadataValue::String("rust".to_string()),
        );

        let mut metadata_3 = HashMap::new();
        metadata_3.insert(
            "document_id".to_string(),
            MetadataValue::String("python".to_string()),
        );

        database
            .insert(Vector {
                id: "rust_chunk_0".to_string(),
                values: vec![1.0, 0.0],
                metadata: metadata_1,
            })
            .unwrap();

        database
            .insert(Vector {
                id: "rust_chunk_1".to_string(),
                values: vec![0.0, 1.0],
                metadata: metadata_2,
            })
            .unwrap();

        database
            .insert(Vector {
                id: "python_chunk_0".to_string(),
                values: vec![1.0, 1.0],
                metadata: metadata_3,
            })
            .unwrap();

        let deleted =
            database.delete_by_metadata("document_id", &MetadataValue::String("rust".to_string()));

        assert_eq!(deleted, 2);

        assert!(database.get("rust_chunk_0").is_none());
        assert!(database.get("rust_chunk_1").is_none());
        assert!(database.get("python_chunk_0").is_some());
    }

    #[test]
    fn test_list_documents() {
        let mut database = Database::new();

        let mut rust_metadata = HashMap::new();
        rust_metadata.insert(
            "document_id".to_string(),
            MetadataValue::String("rust".to_string()),
        );

        let mut rust_metadata_2 = HashMap::new();
        rust_metadata_2.insert(
            "document_id".to_string(),
            MetadataValue::String("rust".to_string()),
        );

        let mut python_metadata = HashMap::new();
        python_metadata.insert(
            "document_id".to_string(),
            MetadataValue::String("python".to_string()),
        );

        database
            .insert(Vector {
                id: "rust_chunk_0".to_string(),
                values: vec![1.0, 0.0],
                metadata: rust_metadata,
            })
            .unwrap();

        database
            .insert(Vector {
                id: "rust_chunk_1".to_string(),
                values: vec![0.0, 1.0],
                metadata: rust_metadata_2,
            })
            .unwrap();

        database
            .insert(Vector {
                id: "python_chunk_0".to_string(),
                values: vec![1.0, 1.0],
                metadata: python_metadata,
            })
            .unwrap();

        let documents = database.list_documents();

        assert_eq!(
            documents,
            vec![("python".to_string(), 1), ("rust".to_string(), 2),]
        );
    }
}
