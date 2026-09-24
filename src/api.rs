use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;

use crate::document::Document;
use crate::engine::RecallEngine;
use crate::metadata::MetadataValue;
use crate::retrieval::{Retriever, SearchOptions};

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<RecallEngine>>,
}

#[derive(Debug, Deserialize)]
pub struct AddDocumentRequest {
    pub id: String,
    pub text: String,

    #[serde(default)]
    pub metadata: HashMap<String, Value>,

    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
}

fn default_chunk_size() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct AddDocumentResponse {
    pub document_id: String,
    pub chunks_indexed: usize,
}

#[derive(Debug, Serialize)]
pub struct DocumentSummary {
    pub document_id: String,
    pub chunks: usize,
}

#[derive(Debug, Serialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentSummary>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,

    #[serde(default = "default_top_k")]
    pub top_k: usize,

    #[serde(default)]
    pub document_id: Option<String>,

    #[serde(default)]
    pub min_score: Option<f32>,
}

fn default_top_k() -> usize {
    3
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<crate::search_result::SearchResult>,
}


pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/documents", get(list_documents).post(add_document))
        .route("/search",post(search))
        .with_state(state)
}

async fn health() -> &'static str {
    "RECALL API is running"
}

async fn add_document(
    State(state): State<AppState>,
    Json(request): Json<AddDocumentRequest>,
) -> Result<Json<AddDocumentResponse>, String> {
    let metadata = json_to_metadata(request.metadata)?;
    
    let document = Document {
        id: request.id,
        text: request.text,
        metadata,
    };

    let document_id = document.id.clone();
    let chunk_size = request.chunk_size;

    let chunks_indexed = tokio::task::spawn_blocking(move || {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        let chunks_indexed = engine
            .add_document(&document, chunk_size)
            .map_err(|error| error.to_string())?;

        engine
            .save("recall.json")
            .map_err(|error| error.to_string())?;

        Ok::<usize, String>(chunks_indexed)
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(Json(AddDocumentResponse {
        document_id,
        chunks_indexed,
    }))
}

async fn search(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, String> {
    let query = request.query;
    let top_k = request.top_k;
    let document_id = request.document_id;
    let min_score = request.min_score;

    let results = tokio::task::spawn_blocking(move || {
        let engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        let options = SearchOptions {
            top_k,
            document_id,
            min_score,
            filters: Vec::new(),
        };

        engine
            .search(&query, options)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(Json(SearchResponse { results }))
}

async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<ListDocumentsResponse>, String> {
    let documents = tokio::task::spawn_blocking(move || {
        let engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        Ok::<Vec<(String, usize)>, String>(engine.list_documents())
    })
    .await
    .map_err(|error| error.to_string())??;

    let documents = documents
        .into_iter()
        .map(|(document_id, chunks)| DocumentSummary {
            document_id,
            chunks,
        })
        .collect();

    Ok(Json(ListDocumentsResponse { documents }))
}

fn json_to_metadata_value(value: Value) -> Result<MetadataValue, String> {
    match value {
        Value::String(value) => Ok(MetadataValue::String(value)),
        Value::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Ok(MetadataValue::Integer(integer))
            } else if let Some(float) = value.as_f64() {
                Ok(MetadataValue::Float(float))
            } else {
                Err("Unsupported number value".to_string())
            }
        }
        Value::Bool(value) => Ok(MetadataValue::Boolean(value)),
        _ => Err("Metadata values must be strings, numbers, or booleans".to_string()),
    }
}

fn json_to_metadata(
    metadata: HashMap<String, Value>,
) -> Result<HashMap<String, MetadataValue>, String> {
    metadata
        .into_iter()
        .map(|(key, value)| {
            Ok((key, json_to_metadata_value(value)?))
        })
        .collect()
}