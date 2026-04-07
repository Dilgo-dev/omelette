use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::{Column, Row};

use crate::core::backend::{Backend, ColumnInfo, SchemaName, TableName};
use crate::core::result::QueryResult;

/// `SQLite` backend powered by `sqlx::SqlitePool`.
pub struct SqliteBackend {
    pool: SqlitePool,
}

impl SqliteBackend {
    pub async fn connect(dsn: &str) -> Result<Self> {
        let pool = SqlitePool::connect(dsn)
            .await
            .with_context(|| format!("connecting to {dsn}"))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl Backend for SqliteBackend {
    async fn list_schemas(&self) -> Result<Vec<SchemaName>> {
        Ok(vec![SchemaName("main".into())])
    }

    async fn list_tables(&self, _schema: Option<&str>) -> Result<Vec<TableName>> {
        let rows = sqlx::query(
            "SELECT name FROM sqlite_master \
             WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .context("listing tables")?;
        Ok(rows
            .into_iter()
            .map(|r| TableName {
                schema: Some("main".into()),
                name: r.get::<String, _>(0),
            })
            .collect())
    }

    async fn list_columns(&self, table: &TableName) -> Result<Vec<ColumnInfo>> {
        let sql = format!("PRAGMA table_info(\"{}\")", table.name);
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("listing columns of {}", table.name))?;
        Ok(rows
            .into_iter()
            .map(|r| ColumnInfo {
                name: r.get::<String, _>(1),
                ty: r.get::<String, _>(2),
            })
            .collect())
    }

    async fn preview_table(&self, table: &TableName, limit: u32) -> Result<QueryResult> {
        let sql = format!("SELECT * FROM \"{}\" LIMIT {}", table.name, limit);
        self.run_query(&sql).await
    }

    async fn run_query(&self, sql: &str) -> Result<QueryResult> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("running query: {sql}"))?;
        Ok(rows_to_result(&rows))
    }
}

fn rows_to_result(rows: &[SqliteRow]) -> QueryResult {
    let columns: Vec<String> = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_owned()).collect())
        .unwrap_or_default();
    let data = rows
        .iter()
        .map(|row| (0..columns.len()).map(|i| cell_to_json(row, i)).collect())
        .collect();
    QueryResult {
        columns,
        rows: data,
    }
}

fn cell_to_json(row: &SqliteRow, i: usize) -> Value {
    if let Ok(Some(s)) = row.try_get::<Option<String>, _>(i) {
        return Value::String(s);
    }
    if let Ok(Some(n)) = row.try_get::<Option<i64>, _>(i) {
        return Value::from(n);
    }
    if let Ok(Some(f)) = row.try_get::<Option<f64>, _>(i) {
        return Value::from(f);
    }
    Value::Null
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_tables_and_query_in_memory() {
        let backend = SqliteBackend::connect("sqlite::memory:")
            .await
            .expect("connect");
        backend
            .run_query("CREATE TABLE pets (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
            .await
            .expect("create");
        backend
            .run_query("INSERT INTO pets (name, age) VALUES ('milo', 4), ('luna', 2)")
            .await
            .expect("insert");

        let tables = backend.list_tables(None).await.expect("list");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "pets");

        let preview = backend
            .preview_table(&tables[0], 10)
            .await
            .expect("preview");
        assert_eq!(preview.columns, vec!["id", "name", "age"]);
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.rows[0][1], Value::String("milo".into()));
        assert_eq!(preview.rows[0][2], Value::from(4_i64));
    }

    #[tokio::test]
    async fn list_schemas_returns_main() {
        let backend = SqliteBackend::connect("sqlite::memory:")
            .await
            .expect("connect");
        let schemas = backend.list_schemas().await.expect("schemas");
        assert_eq!(schemas, vec![SchemaName("main".into())]);
    }
}
