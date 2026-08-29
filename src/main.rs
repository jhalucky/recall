mod database;
mod error;
mod metadata;
mod search_result;
mod similarity;
mod vector;

use std::collections::HashMap;
use std::env;
use std::path::Path;

use crate::database::Database;
use crate::error::RecallError;
use crate::vector::Vector;

fn main() -> Result<(), RecallError> {
    let mut database;

    // Load existing database or create a new one.
    if Path::new("recall.json").exists() {
        database = Database::load("recall.json")?;
    } else {
        database = Database::new();
    }

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Please provide a command.");
        return Ok(());
    }

    match args[1].as_str() {
        "search" => {
            if args.len() < 3 {
                println!("Usage: cargo run -- search <value1> <value2> ... [--top-k N]");
                return Ok(());
            }

            let mut values = Vec::new();
            let mut top_k = 3;
            let mut i = 2;

            while i < args.len() {
                if args[i] == "--top-k" {
                    if i + 1 >= args.len() {
                        println!("Missing value after --top-k");
                        return Ok(());
                    }

                    match args[i + 1].parse::<usize>() {
                        Ok(k) => top_k = k,
                        Err(_) => {
                            println!("Invalid top-k value");
                            return Ok(());
                        }
                    }

                    break;
                }

                match args[i].parse::<f32>() {
                    Ok(value) => values.push(value),
                    Err(_) => {
                        println!("Invalid vector value: {}", args[i]);
                        return Ok(());
                    }
                }

                i += 1;
            }

            let query = values;

            match database.search(&query, top_k) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No results found.");
                    } else {
                        for result in results {
                            println!("{} → {}", result.id, result.score);
                        }
                    }
                }

                Err(RecallError::DimensionMismatch { query, stored }) => {
                    println!(
                        "Search failed: query has {} dimensions, stored vector has {} dimensions",
                        query, stored
                    );
                }

                Err(RecallError::VectorAlreadyExists) => {
                    println!("Vector already exists.");
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
                println!("Usage: cargo run -- insert <id> <value1> <value2> ...");
                return Ok(());
            }

            let id = args[2].clone();

            let values: Result<Vec<f32>, _> =
                args[3..].iter().map(|value| value.parse::<f32>()).collect();

            match values {
                Ok(values) => {
                    let vector = Vector {
                        id,
                        values,
                        metadata: HashMap::new(),
                    };

                    database.insert(vector)?;
                    database.save("recall.json")?;

                    println!("Vector inserted successfully.");
                }

                Err(error) => {
                    println!("Invalid vector value: {}", error);
                }
            }
        }

        "get" => {
            if args.len() < 3 {
                println!("Usage: cargo run -- get <id>");
                return Ok(());
            }

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

        "delete" => {
            if args.len() < 3 {
                println!("Usage: cargo run -- delete <id>");
                return Ok(());
            }

            let id = &args[2];

            match database.delete(id) {
                Some(vector) => {
                    database.save("recall.json")?;
                    println!("Deleted vector: {}", vector.id);
                }

                None => {
                    println!("Vector not found: {}", id);
                }
            }
        }

        "upsert" => {
            if args.len() < 4 {
                println!("Usage: cargo run -- upsert <id> <value1> <value2> ...");
                return Ok(());
            }

            let id = args[2].clone();

            let values: Result<Vec<f32>, _> =
                args[3..].iter().map(|value| value.parse::<f32>()).collect();

            match values {
                Ok(values) => {
                    let vector = Vector {
                        id,
                        values,
                        metadata: HashMap::new(),
                    };

                    database.upsert(vector);
                    database.save("recall.json")?;

                    println!("Vector upserted successfully.");
                }

                Err(error) => {
                    println!("Invalid vector value: {}", error);
                }
            }
        }

        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}
