#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalDiagnostics {
    pub candidates: usize,
    pub results_before_min_score: usize,
    pub results_after_min_score: usize
}