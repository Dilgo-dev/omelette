use serde_json::Value;

/// Unified query result shared by SQL and document backends.
#[derive(Debug, Default, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}
