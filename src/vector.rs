use crate::metadata::MetadataValue;
use std::collections::HashMap;

pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, MetadataValue>,
}
