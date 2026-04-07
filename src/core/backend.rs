use anyhow::Result;
use async_trait::async_trait;

use crate::core::result::QueryResult;

/// A namespace inside a database (a SQL schema, a Mongo database, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaName(pub String);

/// A relation inside a schema (table, view, collection).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableName {
    pub schema: Option<String>,
    pub name: String,
}

/// Common surface of every database backend (SQL or document).
#[async_trait]
pub trait Backend: Send + Sync {
    async fn list_schemas(&self) -> Result<Vec<SchemaName>>;
    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableName>>;
    async fn preview_table(&self, table: &TableName, limit: u32) -> Result<QueryResult>;
    async fn run_query(&self, sql: &str) -> Result<QueryResult>;
}
