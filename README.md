## How to Use

### Requirements

- Rust and Cargo
- Python 3
- A running embedding service

### 1. Clone the repository

```bash
git clone <repository-url>
cd recall
```

### 2. Start the embedding service

Install the Python dependencies:

```bash
pip install -r embedding-service/requirements.txt
```

Start the embedding service:

```bash
python embedding-service/app.py
```

The embedding service runs at:

```text
http://127.0.0.1:8000
```

### 3. Build RECALL

```bash
cargo build --release
```

### 4. Install the RECALL CLI

```bash
cargo install --path . --force
```

After installation, verify it:

```bash
recall --help
```

### 5. Add a document

```bash
recall add-document <path-to-document>
```

Example:

```bash
recall add-document networking.txt
```

### 6. Search documents

```bash
recall search-text "What translates domain names into IP addresses?" --top-k 5
```

You can also restrict the search to a specific document:

```bash
recall search-text "What is DNS?" --document networking --top-k 5
```

### 7. List documents

```bash
recall list-documents
```

### 8. Delete a document

```bash
recall delete-document networking
```

### 9. Run retrieval evaluation

```bash
recall eval
```

### 10. Use RECALL as a Rust library

Add RECALL to another Rust project:

```toml
[dependencies]
recall = { path = "../recall" }
```

Then use the public API:

```rust
use recall::{Document, RecallEngine, SearchOptions};
```

Create the engine:

```rust
let mut engine = RecallEngine::new(
    "http://127.0.0.1:8000".to_string()
);
```

Add documents:

```rust
engine.add_document(&document, 100)?;
```

Search:

```rust
let results = engine.search(
    "What is TCP?",
    SearchOptions {
        top_k: 5,
        document_id: None,
        min_score: Some(0.40),
    },
)?;
```

Save the index:

```rust
engine.save("recall.json")?;
```

Load it later:

```rust
let engine = RecallEngine::load(
    "recall.json",
    "http://127.0.0.1:8000".to_string(),
)?;
```

For a complete library example, see:

```text
examples/basic.rs
```

### Typical Usage

```text
Start embedding service
        ↓
Install RECALL
        ↓
Add documents
        ↓
RECALL chunks + embeds + stores them
        ↓
Ask a question
        ↓
RECALL returns relevant chunks
        ↓
Pass chunks to an LLM if building a RAG application
```