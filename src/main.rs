use std::{collections::HashMap, println};

struct Vector {
    id: String,
    values: Vec<f32>,
}

struct SearchResult {
    id: String,
    score: f32,
}

struct Database {
    vectors: HashMap<String, Vector>,
}

impl Database {
    fn new() -> Database {
        Database {
            vectors: HashMap::new(),
        }
    }

    fn insert(&mut self, vector: Vector) {
        self.vectors.insert(vector.id.clone(), vector);
    }

    fn get(&self, id: &str) -> Option<&Vector> {
        self.vectors.get(id)
    }

    fn search(&self, query: &[f32]) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for vector in self.vectors.values() {
            let score = cosine_similarity(query, &vector.values);

            results.push(SearchResult {
                id: vector.id.clone(),
                score,
            });
        }
        results
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut magnitude_a = 0.0;
    let mut magnitude_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];

        magnitude_a += a[i] * a[i];
        magnitude_b += b[i] * b[i];
    }

    dot_product / (magnitude_a.sqrt() * magnitude_b.sqrt())
}
fn main() {
    let mut database = Database::new();

    let vector = Vector {
        id: String::from("doc_001"),
        values: vec![0.12, 0.55, 0.81],
    };

    let vector2 = Vector {
        id: String::from("doc_002"),
        values: vec![0.91, 0.12, 0.44],
    };

    let vector3 = Vector {
        id: String::from("doc_003"),
        values: vec![0.33, 0.72, 0.48],
    };

    database.insert(vector);
    database.insert(vector2);
    database.insert(vector3);

    let query = vec![0.10, 0.51, 0.79];
    let results = database.search(&query);

    for result in results {
        println!("{} * {}", result.id, result.score);
    }
}
