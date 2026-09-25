use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::document::Document;
use crate::document::DocumentPage;
use crate::engine::RecallEngine;
use crate::filter::MetadataFilter;
use crate::metadata::MetadataValue;
use crate::retrieval::SearchOptions;

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

#[derive(Debug, Deserialize)]
struct DocumentPageRequest {
    page: usize,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AddPagesRequest {
    id: String,
    pages: Vec<DocumentPageRequest>,
    metadata: HashMap<String, serde_json::Value>,
    #[serde(default = "default_chunk_size")]
    chunk_size: usize,
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

    #[serde(default)]
    pub filters: Vec<MetadataFilterRequest>,
}

#[derive(Debug, Deserialize)]
pub struct MetadataFilterRequest {
    pub key: String,
    pub value: Value,
}

fn default_top_k() -> usize {
    3
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<crate::search_result::SearchResult>,
}

#[derive(Debug, Serialize)]
pub struct DeleteDocumentResponse {
    pub document_id: String,
    pub chunks_deleted: usize,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/documents", get(list_documents).post(add_document))
        .route("/documents/{document_id}", delete(delete_document))
        .route("/search", post(search))
        .route("/documents/pages", post(add_document_pages))
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
    let document_id = request.id;

    let document = Document {
        id: document_id.clone(),
        text: request.text,
        metadata,
    };

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

    let filters: Vec<MetadataFilter> = request
        .filters
        .into_iter()
        .map(|filter| {
            Ok(MetadataFilter::new(
                filter.key,
                json_to_metadata_value(filter.value)?,
            ))
        })
        .collect::<Result<_, String>>()?;

    let results = tokio::task::spawn_blocking(move || {
        let engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        let options = SearchOptions {
            top_k,
            document_id,
            min_score,
            filters,
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
        .map(|(key, value)| Ok((key, json_to_metadata_value(value)?)))
        .collect()
}

async fn add_document_pages(
    State(state): State<AppState>,
    Json(request): Json<AddPagesRequest>,
) -> Result<Json<AddDocumentResponse>, String> {
    let metadata = json_to_metadata(request.metadata)?;

    let pages: Vec<DocumentPage> = request
        .pages
        .into_iter()
        .map(|page| DocumentPage {
            page_number: page.page,
            text: page.text,
        })
        .collect();

    let document_id = request.id;
    let chunk_size = request.chunk_size;
    let document_id_for_engine = document_id.clone();

    let chunks_indexed = tokio::task::spawn_blocking(move || {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        let result = engine
            .add_document_pages(&document_id_for_engine, &pages, &metadata, chunk_size)
            .map_err(|error| error.to_string())?;

        engine
            .save("recall.json")
            .map_err(|error| error.to_string())?;

        Ok::<usize, String>(result)
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(Json(AddDocumentResponse {
        document_id,
        chunks_indexed,
    }))
}

async fn delete_document(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<Json<DeleteDocumentResponse>, String> {
    let document_id_for_engine = document_id.clone();

    let chunks_deleted = tokio::task::spawn_blocking(move || {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Failed to lock RECALL engine".to_string())?;

        let chunks_deleted = engine.delete_document(&document_id_for_engine);

        engine
            .save("recall.json")
            .map_err(|error| error.to_string())?;

        Ok::<usize, String>(chunks_deleted)
    })
    .await
    .map_err(|error| error.to_string())??;

    Ok(Json(DeleteDocumentResponse {
        document_id,
        chunks_deleted,
    }))
}
