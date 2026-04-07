use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use omelette::core::engine::Engine;

/// A user-defined database connection saved to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub label: String,
    pub engine: Engine,
    pub dsn: String,
}

#[allow(dead_code)] // CRUD API consumed by the connections list UI in #5
impl Connection {
    pub fn new(label: impl Into<String>, engine: Engine, dsn: impl Into<String>) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let pid = std::process::id();
        Self {
            id: format!("{pid:x}{nanos:x}"),
            label: label.into(),
            engine,
            dsn: dsn.into(),
        }
    }
}

/// In-memory store of saved connections, persisted as JSON.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConnectionStore {
    #[serde(default)]
    pub connections: Vec<Connection>,
}

impl ConnectionStore {
    pub fn load() -> Result<Self> {
        let path = store_path()?;
        if !path.exists() {
            let store = Self::default();
            store.save()?;
            return Ok(store);
        }
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let store: Self =
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let path = store_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        write_atomic(&path, json.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)] // wired in #5
    pub fn add(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    #[allow(dead_code)] // wired in #5
    pub fn remove(&mut self, id: &str) -> Option<Connection> {
        let idx = self.connections.iter().position(|c| c.id == id)?;
        Some(self.connections.remove(idx))
    }

    #[allow(dead_code)] // wired in #5
    pub fn update(&mut self, id: &str, update: impl FnOnce(&mut Connection)) -> bool {
        self.connections
            .iter_mut()
            .find(|c| c.id == id)
            .is_some_and(|c| {
                update(c);
                true
            })
    }

    #[allow(dead_code)] // wired in #5
    pub fn get(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }
}

fn store_path() -> Result<PathBuf> {
    let dirs =
        ProjectDirs::from("", "", "omelette").context("resolving omelette config directory")?;
    Ok(dirs.config_dir().join("connections.json"))
}

fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("path has no parent")?;
    let tmp = tempfile_in(parent);
    {
        let mut f =
            fs::File::create(&tmp).with_context(|| format!("creating {}", tmp.display()))?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

fn tempfile_in(dir: &std::path::Path) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    dir.join(format!(".connections.json.{pid}.{nanos}.tmp"))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_round_trip() {
        let mut store = ConnectionStore::default();
        let c = Connection::new("local", Engine::Sqlite, "sqlite::memory:");
        let id = c.id.clone();
        store.add(c);
        assert_eq!(store.connections.len(), 1);
        assert!(store.get(&id).is_some());
        store.remove(&id);
        assert!(store.connections.is_empty());
    }

    #[test]
    fn update_mutates_in_place() {
        let mut store = ConnectionStore::default();
        let c = Connection::new("a", Engine::Sqlite, "sqlite::memory:");
        let id = c.id.clone();
        store.add(c);
        let ok = store.update(&id, |c| c.label = "renamed".into());
        assert!(ok);
        let got = store.get(&id).expect("connection should exist");
        assert_eq!(got.label, "renamed");
    }
}
