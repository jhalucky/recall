use recall::{Document, EmbeddingConfig, RecallEngine, SearchOptions};
use std::{collections::HashMap, println};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut recall = RecallEngine::new(
        "http://127.0.0.1:8000".to_string(),
        EmbeddingConfig {
            provider: "sentence-transformers".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            version: "1".to_string(),
        },
    );

    let document = Document {
        id: "networking".to_string(),
        text: "DNS translates human-readable domain names into IP addresses.".to_string(),
        metadata: HashMap::new(),
    };

    recall.add_document(&document, 100)?;

    let results = recall.search(
        "What translates domain names into IP addresses?",
        SearchOptions {
            top_k: 3,
            document_id: None,
            min_score: Some(0.3),
        },
    )?;

    for result in results {
        println!("{} ({})", result.text, result.score);
    }

    Ok(())
}
