mod chunker;
mod database;
mod document;
mod embedding;
mod error;
mod metadata;
mod pipeline;
mod search_result;
mod similarity;
mod tokenizer;
mod vector;

use std::collections::HashMap;
use std::path::Path;
use std::{env, println};

use crate::database::Database;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
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

                Err(RecallError::ReqwestError(error)) => {
                    println!("Embedding service error: {}", error);
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

        "add-document" => {
            if args.len() < 3 {
                println!("Usage: cargo run -- add-document <file>");
                return Ok(());
            }

            let path = &args[2];

            let text = match std::fs::read_to_string(path) {
                Ok(text) => text,
                Err(error) => {
                    println!("Failed to read document: {}", error);
                    return Ok(());
                }
            };

            let document_id = Path::new(path)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("document")
                .to_string();

            let doc = document::Document {
                id: document_id,
                text,
                metadata: HashMap::new(),
            };

            let embedder = embedding::EmbeddingClient::new("http://127.0.0.1:8000".to_string());

            let inserted = pipeline::process_document(&doc, 20, &embedder, &mut database)?;

            database.save("recall.json")?;

            println!("Document added successfully.");
            println!("Created {} vector(s).", inserted);
        }

        "search-text" => {
            if args.len() < 3 {
                println!("Usage: cargo run -- search-text <query> [--top-k N] [--document NAME]");
                return Ok(());
            }

            let mut query_parts = Vec::new();
            let mut top_k = 3;
            let mut document_filter: Option<String> = None;

            let mut i = 2;

            while i < args.len() {
                if args[i] == "--top-k" {
                    if i + 1 >= args.len() {
                        println!("Missing value after --top-k");
                        return Ok(());
                    }

                    match args[i + 1].parse::<usize>() {
                        Ok(k) if k > 0 => top_k = k,
                        _ => {
                            println!("Invalid top-k value");
                            return Ok(());
                        }
                    }

                    i += 2;
                    continue;
                }

                if args[i] == "--document" {
                    if i + 1 >= args.len() {
                        println!("Missing document name after --document");
                        return Ok(());
                    }

                    document_filter = Some(args[i + 1].clone());

                    i += 2;
                    continue;
                }

                query_parts.push(args[i].clone());
                i += 1;
            }

            if query_parts.is_empty() {
                println!("Search query cannot be empty.");
                return Ok(());
            }

            let query = query_parts.join(" ");

            let embedder = embedding::EmbeddingClient::new("http://127.0.0.1:8000".to_string());

            let query_vector = embedder.embed(&query)?;

            let results = match document_filter {
                Some(document_id) => database.search_with_filter(
                    &query_vector,
                    top_k,
                    "document_id",
                    &MetadataValue::String(document_id),
                )?,
                None => database.search(&query_vector, top_k)?,
            };

            if results.is_empty() {
                println!("No results found.");
            } else {
                println!("Search results for: \"{}\"", query);

                for result in results {
                    println!("{} -> {}", result.id, result.score);
                    println!(" {}", result.text);
                    println!();
                }
            }
        }

        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}
