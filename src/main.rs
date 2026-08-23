mod database;
mod error;
mod metadata;
mod search_result;
mod similarity;
mod vector;

use std::vec;

use crate::database::Database;
use crate::error::RecallError;
use crate::vector::Vector;

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
                query, stored
            );
        }
    }
}
