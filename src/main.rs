mod database;
mod error;
mod metadata;
mod search_result;
mod similarity;
mod vector;

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::{println, vec};

use crate::database::Database;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::vector::Vector;

fn main() -> Result<(), RecallError> {
    let mut database;

    if Path::new("recall.json").exists() {
        database = Database::load("recall.json")?;
    } else {
        database = Database::new();
    }

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "search" => {
                let query = vec![1.0, 0.0];
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
                if args.len() < 4 {
                    println!("Usage: cargo run -- insert <id> <value1> <value2> ...")
                } else {
                    let id = args[2].clone();

                    let values: Result<Vec<f32>, _> = args[3..]
                        .iter()
                        .map(|value| value.parse::<f32>())
                        .collect();

                    match values {
                        Ok(values) => {
                            let vector = Vector {
                                id,
                                values,
                                metadata: HashMap::new()
                            };

                            database.insert(vector)?;
                            database.save("recall.json")?;

                            println!("Vector inserted successfully.")
                        }

                        Err(error) => {
                            println!("invalid vector values: {}", error);
                        }
                    }
                }

                
            }
            "get" => {
                if args.len() < 3 {
                    println!("Usage: cargo run -- get <id>");                
                } else {
                    let id = &args[2];

                    match database.get(id) {
                        Some(vector) => {
                            println!("ID: {}", vector.id);
                            println!("Vector: {:?}", vector.values);
                            println!("Metadata: {:?}", vector.metadata);
                        }

                        None => {
                            println!("Vector not found: {}", id);
                        }
                    }
                }
            }
            "delete" => {
                if args.len() < 3 {
                    println!("usage: cargo run -- delete <id>")
                } else {
                    let id = &args[2];

                    match database.delete(id) {
                        Some(vector) => {
                            database.save("recall.json")?;

                            println!("Deleted vector: {}", vector.id);
                        }

                        None => {
                            println!("vector not found: {}", id);
                        }
                    }
                }
            }
            "upsert" => {
                let vector = Vector {
                    id: String::from("doc_004"),
                    values: vec![0.4, 1.5],
                    metadata: HashMap::new()
                };

                database.upsert(vector);
                database.save("recall.json")?;

                println!("Vector upserted successfully!")
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

    Ok(())
}
