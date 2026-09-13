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


#[cfg(test)]
mod tests {
    use crate::filter;

use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_filter_matches_metadata() {
        let mut metadata = HashMap::new();

        metadata.insert(
            "category".to_string(),
            MetadataValue::String("notes".to_string())
        );

        let filter = MetadataFilter::new(
            "category".to_string(),
            MetadataValue::String("notes".to_string())
        );

        assert!(filter.matches(&metadata));
    }

    #[test]
    fn test_filter_rejects_different_value() {
        let mut metadata = HashMap::new();

        metadata.insert(
            "category".to_string(), 
            MetadataValue::String("notes".to_string())
        );

        let filter = MetadataFilter::new(
              "category".to_string(),
              MetadataValue::String("notes".to_string())
        );

        assert!(filter.matches(&metadata))
    }

    #[test]
    fn test_filter_rejects_missing_key() {
        let metadata = HashMap::new();


        let filter = MetadataFilter::new(
            "category".to_string(),
            MetadataValue::String("notes".to_string())
        );

        assert!(!filter.matches(&metadata));
    }


}