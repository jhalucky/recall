use std::{collections::HashMap, fs};

use crate::metadata::MetadataValue;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub text: String,
    pub metadata: HashMap<String, MetadataValue>,
}

pub fn load_from_file(path: &str) -> Result<Document, std::io::Error> {
    let text = fs::read_to_string(path)?;

    let id = Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("document")
        .to_string();

    Ok(Document {
        id,
        text,
        metadata: HashMap::new(),
    })
}
