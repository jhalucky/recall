pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

#[cfg(test)]

mod tests {
    use std::{assert_eq, vec};

    use super::*;

    #[test]
    fn test_tokenize_text() {
        let text = "Rust is fast and memory safe";

        let tokens = tokenize(text);

        assert_eq!(tokens, vec!["Rust", "is", "fast", "and", "memory", "safe"]);
    }
}
