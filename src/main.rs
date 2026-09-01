mod chunker;
mod database;
mod document;
mod embedding;
mod error;
mod evaluation;
mod metadata;
mod pipeline;
mod search_result;
mod similarity;
mod tokenizer;
mod vector;

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::Path;

use crate::database::Database;
use crate::error::RecallError;
use crate::metadata::MetadataValue;
use crate::vector::Vector;

#[derive(Parser, Debug)]
#[command(name = "recall")]
#[command(about = "A semantic document retrieval engine")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search using a vector
    Search {
        /// Vector values
        values: Vec<f32>,

        /// Number of results to return
        #[arg(long, default_value_t = 3)]
        top_k: usize,
    },

    /// Search documents using natural language
    SearchText {
        /// Text query
        query: String,

        /// Number of results to return
        #[arg(long, default_value_t = 3)]
        top_k: usize,

        /// Restrict search to a document
        #[arg(long)]
        document: Option<String>,
    },

    /// Add a document to RECALL
    AddDocument {
        /// Path to the document
        path: String,
    },

    /// List indexed documents
    ListDocuments,

    /// Delete a document and all its chunks
    DeleteDocument {
        /// Document ID
        document_id: String,
    },

    /// Insert a vector
    Insert { id: String, values: Vec<f32> },

    /// Get a vector
    Get { id: String },

    /// Delete a vector
    Delete { id: String },

    /// Upsert a vector
    Upsert { id: String, values: Vec<f32> },

    /// Evaluate retrieval quality
    Eval {
        /// Path to evaluation queries
        #[arg(long, default_value = "eval/queries.json")]
        queries: String,

        /// Number of results considered relevant
        #[arg(long, default_value_t = 3)]
        top_k: usize,
    },
}

fn main() -> Result<(), RecallError> {
    let mut database;

    // Load existing database or create a new one.
    if Path::new("recall.json").exists() {
        database = Database::load("recall.json")?;
    } else {
        database = Database::new();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Search { values, top_k } => match database.search(&values, top_k) {
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
        },

        Commands::Insert { id, values } => {
            let vector = Vector {
                id,
                values,
                metadata: HashMap::new(),
            };

            database.insert(vector)?;
            database.save("recall.json")?;

            println!("Vector inserted successfully.");
        }

        Commands::Get { id } => match database.get(&id) {
            Some(vector) => {
                println!("ID: {}", vector.id);
                println!("Vector: {:?}", vector.values);
                println!("Metadata: {:?}", vector.metadata);
            }

            None => {
                println!("Vector not found: {}", id);
            }
        },

        Commands::Delete { id } => match database.delete(&id) {
            Some(vector) => {
                database.save("recall.json")?;

                println!("Deleted vector: {}", vector.id);
            }

            None => {
                println!("Vector not found: {}", id);
            }
        },

        Commands::DeleteDocument { document_id } => {
            let deleted = database
                .delete_by_metadata("document_id", &MetadataValue::String(document_id.clone()));

            if deleted == 0 {
                println!("Document not found: {}", document_id);
            } else {
                database.save("recall.json")?;

                println!(
                    "Deleted document '{}' and {} chunk(s).",
                    document_id, deleted
                );
            }
        }

        Commands::ListDocuments => {
            let documents = database.list_documents();

            if documents.is_empty() {
                println!("No documents found.");
            } else {
                println!("Documents:");

                for (document_id, chunk_count) in documents {
                    println!("{} → {} chunk(s)", document_id, chunk_count);
                }
            }
        }

        Commands::Upsert { id, values } => {
            let vector = Vector {
                id,
                values,
                metadata: HashMap::new(),
            };

            database.upsert(vector);
            database.save("recall.json")?;

            println!("Vector upserted successfully!");
        }

        Commands::AddDocument { path } => {
            let document = document::load_from_file(&path)?;

            let embedder = embedding::EmbeddingClient::new("http://127.0.0.1:8000".to_string());

            let inserted = pipeline::process_document(&document, 100, &embedder, &mut database)?;

            database.save("recall.json")?;

            println!(
                "Document '{}' added successfully. {} chunk(s) indexed.",
                document.id, inserted
            );
        }

        Commands::SearchText {
            query,
            top_k,
            document,
        } => {
            let embedder = embedding::EmbeddingClient::new("http://127.0.0.1:8000".to_string());

            let query_vector = embedder.embed(&query)?;

            let results = match document {
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
                    println!(" Document: {}", result.document_id);
                    println!(" Chunk: {}", result.chunk_index);
                    println!(" {}", result.text);
                    println!();
                }
            }
        }

        Commands::Eval { queries, top_k } => {
            let evaluation_queries = evaluation::load_queries(&queries)?;

            if evaluation_queries.is_empty() {
                println!("No evaluation queries found.");
                return Ok(());
            }

            let embedder = embedding::EmbeddingClient::new("http://127.0.0.1:8000".to_string());

            let (top_1_correct, top_k_correct) =
                evaluation::evaluate_detailed(&database, &embedder, &evaluation_queries, top_k)?;

            let total = evaluation_queries.len();

            let top_1_accuracy = (top_1_correct as f32 / total as f32) * 100.0;

            let top_k_accuracy = (top_k_correct as f32 / total as f32) * 100.0;

            println!("RECALL Retrieval Evaluation");
            println!();
            println!("Queries: {}", total);
            println!("Top-1 Accuracy: {:.1}%", top_1_accuracy);
            println!("Top-{} Accuracy: {:.1}%", top_k, top_k_accuracy);
            println!();
            println!("Top-1: {}/{}", top_1_correct, total);
            println!("Top-{}: {}/{}", top_k, top_k_correct, total);
        }
    }

    Ok(())
}
