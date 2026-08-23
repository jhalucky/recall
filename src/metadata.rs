#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_metadata_values() {
        let title = MetadataValue::String(String::from("Learning Rust"));
        let year = MetadataValue::Integer(2026);
        let rating = MetadataValue::Float(4.8);
        let published = MetadataValue::Boolean(true);

        assert_eq!(title, MetadataValue::String(String::from("Learning Rust")));

        assert_eq!(year, MetadataValue::Integer(2026));

        assert_eq!(rating, MetadataValue::Float(4.8));

        assert_eq!(published, MetadataValue::Boolean(true));
    }
}
