use crate::error::RecallError;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, RecallError> {
    if a.len() != b.len() {
        return Err(RecallError::DimensionMismatch {
            query: a.len(),
            stored: b.len(),
        });
    }
    let mut dot_product = 0.0;
    let mut magnitude_a = 0.0;
    let mut magnitude_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];

        magnitude_a += a[i] * a[i];
        magnitude_b += b[i] * b[i];
    }

    Ok(dot_product / (magnitude_a.sqrt() * magnitude_b.sqrt()))
}

#[cfg(test)]
mod tests {
    use std::matches;

    use super::*;

    #[test]
    fn test_cosine_similarity_same_vector() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];

        let result = cosine_similarity(&a, &b).unwrap();

        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_opposite_vector() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];

        let result = cosine_similarity(&a, &b).unwrap();

        assert!((result - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_dimension_mismatch() {
        let a = vec![1.0, 3.0, 2.0];
        let b = vec![1.0, 2.0];

        let result = cosine_similarity(&a, &b);

        assert!(matches!(
            result,
            Err(RecallError::DimensionMismatch {
                query: 3,
                stored: 2
            })
        ));
    }
}
