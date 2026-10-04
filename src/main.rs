use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use recall::api::create_router;
use recall::embedding;
use recall::error::RecallError;
use recall::evaluation;
use recall::metadata::MetadataValue;
use recall::pipeline;
use recall::vector::Vector;
use recall::EmbeddingConfig;
use recall::RecallEngine;
use recall::{Database, Retriever, SearchOptions};

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

    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

fn embedding_url() -> String {
    std::env::var("RECALL_EMBEDDING_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8001".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn data_path() -> String {
    std::env::var("RECALL_DATA_PATH").unwrap_or_else(|_| "recall.json".to_string())
}

#[tokio::main]
async fn main() -> Result<(), RecallError> {
    let mut database;

    let database_path = data_path();

    // Load existing database or create a new one.
    if Path::new(&database_path).exists() {
        database = Database::load(&database_path)?;
    } else {
        database = Database::new(EmbeddingConfig {
            provider: "sentence-transformers".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            version: "1".to_string(),
        });
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Search { values, top_k } => match database.search(&values, top_k) {
            Ok(results) => {
                if results.is_empty() {
                    println!("No results found.");
                } else {
                    for result in results {
                        println!("{} → {}", result.chunk_id, result.score);
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
            database.save(&database_path)?;

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
                database.save(&database_path)?;

                println!("Deleted vector: {}", vector.id);
            }

            None => {
                println!("Vector not found: {}", id);
            }
        },

        Commands::DeleteDocument { document_id } => {
            let deleted = database.delete_by_metadata(
                "document_id",
                &MetadataValue::String(document_id.clone()),
            );

            if deleted == 0 {
                println!("Document not found: {}", document_id);
            } else {
                database.save(&database_path)?;

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

            database.upsert(vector)?;
            database.save(&database_path)?;

            println!("Vector upserted successfully!");
        }

        Commands::AddDocument { path } => {
            let document = recall::load_from_file(&path)?;

            let embedder = embedding::EmbeddingClient::new(embedding_url());

            let inserted = pipeline::process_document(&document, 100, &embedder, &mut database)?;

            database.save(&database_path)?;

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
            let embedder = embedding::EmbeddingClient::new(embedding_url());

            let retriever = Retriever::new(&database, &embedder);

            let options = SearchOptions {
                top_k,
                document_id: document,
                min_score: None,
                filters: Vec::new(),
            };

            let results = retriever.search(&query, options)?;

            if results.is_empty() {
                println!("No results found.");
            } else {
                println!("Search results for: \"{}\"", query);

                for result in results {
                    println!("{} -> {}", result.chunk_id, result.score);
                    println!(" Document: {}", result.document_id);
                    println!(" Chunk: {}", result.chunk_index);
                    println!(" {}", result.text);
                    println!();
                }
            }
        }

        Commands::Serve { host, port } => {
            let embedding_service_url = embedding_url();

            let engine = if Path::new(&database_path).exists() {
                RecallEngine::load(&database_path, embedding_service_url)?
            } else {
                RecallEngine::new(
                    embedding_service_url,
                    EmbeddingConfig {
                        provider: "sentence-transformers".to_string(),
                        model: "all-MiniLM-L6-v2".to_string(),
                        dimension: 384,
                        version: "1".to_string(),
                    },
                )
            };

            let state = recall::api::AppState {
                engine: std::sync::Arc::new(std::sync::Mutex::new(engine)),
            };

            let app = create_router(state);

            let address: SocketAddr = format!("{}:{}", host, port).parse().map_err(|error| {
                RecallError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
            })?;

            println!("RECALL API listening on http://{}", address);

            let listener = tokio::net::TcpListener::bind(address)
                .await
                .map_err(RecallError::IoError)?;

            axum::serve(listener, app)
                .await
                .map_err(RecallError::IoError)?;
        }

        Commands::Eval { queries, top_k } => {
            let evaluation_queries = evaluation::load_queries(&queries)?;

            if evaluation_queries.is_empty() {
                println!("No evaluation queries found.");
                return Ok(());
            }

            let total = evaluation_queries.len();

            let result = tokio::task::spawn_blocking(move || {
                let embedder = embedding::EmbeddingClient::new(embedding_url());

                evaluation::evaluate_detailed(&database, &embedder, &evaluation_queries, top_k)
            })
            .await
            .map_err(|error| RecallError::IoError(std::io::Error::other(error.to_string())))??;

            let (top_1_correct, top_k_correct, mrr) = result;

            let top_1_accuracy = (top_1_correct as f32 / total as f32) * 100.0;
            let top_k_accuracy = (top_k_correct as f32 / total as f32) * 100.0;

            println!();
            println!("MRR: {:.3}", mrr);
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
