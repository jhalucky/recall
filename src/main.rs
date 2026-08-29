mod database;
mod error;
mod metadata;
mod search_result;
mod similarity;
mod vector;

use std::collections::HashMap;
use std::{println, vec};

use crate::database::Database;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::vector::Vector;

fn main() {
    let mut database = Database::new();

    let mut metadata = HashMap::new();

    metadata.insert(
        String::from("title"),
        MetadataValue::String(String::from("Learning Rust")),
    );
    metadata.insert(String::from("year"), MetadataValue::Integer(2026));
    metadata.insert(String::from("Published"), MetadataValue::Boolean(true));

    let vector = Vector {
        id: String::from("doc_001"),
        values: vec![0.12, 0.55, 0.81],
        metadata,
    };

    let vector2 = Vector {
        id: String::from("doc_002"),
        values: vec![0.91, 0.12, 0.44],
        metadata: HashMap::new(),
    };

    let vector3 = Vector {
        id: String::from("doc_003"),
        values: vec![0.33, 0.72, 0.48],
        metadata: HashMap::new(),
    };

    database.insert(vector);
    database.insert(vector2);
    database.insert(vector3);

    let query = vec![0.12, 0.55, 0.81];
    match database.search(&query, 1) {
        Ok(results) => {
            for result in results {
                println!("{} → {}", result.id, result.score);
            }
        }

        Err(RecallError::DimensionMismatch { query, stored }) => {
            println!(
                "Search failed: query has {} dimensions, stored vector has {} dimensions",
                query, stored
            );
        }

        Err(RecallError::VectorAlreadyExists) => {
            println!("VectorAlreadyExists");
        }
    }
}
