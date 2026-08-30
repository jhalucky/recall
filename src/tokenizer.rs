use std::collections::HashMap;

pub struct Tokenizer {
    vocabulary: HashMap<String, usize>,
}

impl Tokenizer {
    pub fn new(vocabulary: HashMap<String, usize>) -> Self {
        Self { vocabulary }
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.split_whitespace()
            .filter_map(|word| self.vocabulary.get(word).copied())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use crate::tokenizer;

    use super::*;

    #[test]
    fn test_encode_text() {
        let mut vocabulary = HashMap::new();

        vocabulary.insert("Rust".to_string(), 1);
        vocabulary.insert("is".to_string(), 2);
        vocabulary.insert("fast".to_string(), 3);

        let tokenizer = Tokenizer::new(vocabulary);

        let tokens = tokenizer.encode("Rust is fast");

        assert_eq!(tokens, vec![1, 2, 3]);
    }
}
