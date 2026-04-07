use serde::{Deserialize, Serialize};

/// Supported database engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Engine {
    Sqlite,
    Postgres,
    Mysql,
    Mongo,
}

impl Engine {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "Postgres",
            Self::Mysql => "MySQL",
            Self::Mongo => "MongoDB",
        }
    }
}
