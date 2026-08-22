pub enum RecallError {
    DimensionMismatch { query: usize, stored: usize },
}
