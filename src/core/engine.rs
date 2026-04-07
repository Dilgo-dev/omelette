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
    pub const ALL: [Self; 4] = [Self::Sqlite, Self::Postgres, Self::Mysql, Self::Mongo];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "Postgres",
            Self::Mysql => "MySQL",
            Self::Mongo => "MongoDB",
        }
    }

    pub const fn badge(self) -> &'static str {
        match self {
            Self::Sqlite => "SQL",
            Self::Postgres => "PG ",
            Self::Mysql => "MY ",
            Self::Mongo => "MON",
        }
    }
}
