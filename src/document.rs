use std::collections::HashMap;

use crate::metadata::MetadataValue;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub text: String,
    pub metadata: HashMap<String, MetadataValue>,
}
