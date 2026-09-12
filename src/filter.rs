use crate::metadata::MetadataValue;

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFilter {
    pub key: String,
    pub value: MetadataValue
}

impl MetadataFilter {
    pub fn new(key: String, value: MetadataValue) -> Self {
        Self {key, value}
    }

    pub fn matches(&self, metadata: &std::collections::HashMap<String, MetadataValue>) -> bool {
        metadata.get(&self.key) == Some(&self.value)
    }
}