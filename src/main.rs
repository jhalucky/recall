mod database;
mod error;
mod metadata;
mod search_result;
mod similarity;
mod vector;

use std::collections::HashMap;
use std::env;
use std::{println, vec};

use crate::database::Database;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::vector::Vector;

fn main() {
    let mut database = Database::new();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "search" => {
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

                    Err(RecallError::IoError(error)) => {
                        println!("I/O error: {}", error);
                    }

                    Err(RecallError::SerializationError(error)) => {
                        println!("Serialization error: {}", error);
                    }
                }
            }

            "insert" => {
                println!("Running insert...")
            }
            "get" => {
                println!("Running get...")
            }
            "delete" => {
                println!("Running delete...")
            }
            "upsert" => {
                println!("Running upsert...")
            }
            _ => {
                println!("Unknown command")
            }
        }
    } else {
        println!("Please provide a command.")
    }

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

    database.insert(vector).unwrap();
    database.insert(vector2).unwrap();
    database.insert(vector3).unwrap();

    let query = vec![0.12, 0.55, 0.81];
    // match database.search(&query, 1) {
    //     Ok(results) => {
    //         for result in results {
    //             // println!("{} → {}", result.id, result.score);
    //         }
    //     }

    //     Err(RecallError::DimensionMismatch { query, stored }) => {
    //         println!(
    //             "Search failed: query has {} dimensions, stored vector has {} dimensions",
    //             query, stored
    //         );
    //     }

    //     Err(RecallError::VectorAlreadyExists) => {
    //         println!("VectorAlreadyExists");
    //     }
    // }
}
