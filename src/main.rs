use std::{collections::HashMap, println};

struct Vector {
    id: String,
    values: Vec<f32>
}

fn main() {
    let mut storage: HashMap<String, Vector> = HashMap::new();

    let vector= Vector{
        id: String::from("doc_001"),
        values: vec![0.12, 0.55, 0.81]
    };

    let vector2 = Vector{
        id: String::from("doc_002"),
        values: vec![0.91, 0.12, 0.44]
    };

    let vector3 = Vector{
        id: String::from("doc_003"),
        values: vec![0.33, 0.72, 0.48]
    };
    
    storage.insert(vector.id.clone(), vector);
    storage.insert(vector2.id.clone(), vector2);
    storage.insert(vector3.id.clone(), vector3);
    println!("Vectors stored: {}",storage.len());
    if let Some(vector) = storage.get("doc_001") {
        println!("Found: {}",vector.id)
    }
}
