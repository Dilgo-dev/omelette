use std::time::Instant;

use anyhow::Result;
use tokio::runtime::{Builder, Runtime};

use crate::connections::{Connection, ConnectionStore};
use directories::ProjectDirs;
use omelette::core::backend::{Backend, TableName};
use omelette::core::engine::Engine;
use omelette::core::result::QueryResult;
use omelette::core::sqlite::SqliteBackend;

fn marker_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("", "", "omelette").map(|d| d.config_dir().join(".bootstrapped"))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Goto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub query: String,
    pub result: Option<QueryResult>,
    pub error: Option<String>,
    pub duration_ms: u128,
    pub status: CellStatus,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum GotoFocus {
    #[default]
    Connections,
    Tables,
}

pub const SPLASH_SLOW: u32 = 70;
pub const SPLASH_FAST: u32 = 25;

pub struct App {
    pub should_quit: bool,
    pub connections: ConnectionStore,
    pub selected: usize,
    pub mode: Mode,
    pub status: Option<String>,
    pub splash_frames: u32,
    pub splash_total: u32,

    pub tables: Vec<TableName>,
    pub loaded_id: Option<String>,

    pub cells: Vec<Cell>,
    pub active_query: String,
    pub scroll_offset: u16,

    pub goto_focus: GotoFocus,
    pub goto_conn_idx: usize,
    pub goto_table_idx: usize,

    backend: Option<Box<dyn Backend>>,
    rt: Runtime,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("should_quit", &self.should_quit)
            .field("selected", &self.selected)
            .field("cells", &self.cells.len())
            .finish_non_exhaustive()
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let rt = Builder::new_current_thread().enable_all().build()?;
        let first_launch = !marker_path().is_some_and(|p| p.exists());
        let splash_total = if std::env::var("OMELETTE_NO_SPLASH").is_ok() {
            0
        } else if first_launch {
            SPLASH_SLOW
        } else {
            SPLASH_FAST
        };
        if first_launch && let Some(p) = marker_path() {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&p, b"1");
        }
        Ok(Self {
            should_quit: false,
            connections: ConnectionStore::load()?,
            selected: 0,
            mode: Mode::Normal,
            status: None,
            splash_frames: splash_total,
            splash_total,
            tables: Vec::new(),
            loaded_id: None,
            cells: Vec::new(),
            active_query: String::new(),
            scroll_offset: 0,
            goto_focus: GotoFocus::Connections,
            goto_conn_idx: 0,
            goto_table_idx: 0,
            backend: None,
            rt,
        })
    }

    pub const fn tick_splash(&mut self) {
        if self.splash_frames > 0 {
            self.splash_frames -= 1;
        }
    }

    pub const fn dismiss_splash(&mut self) {
        self.splash_frames = 0;
    }

    pub const fn splash_active(&self) -> bool {
        self.splash_frames > 0
    }

    pub fn active_push(&mut self, c: char) {
        self.active_query.push(c);
    }

    pub fn active_pop(&mut self) {
        self.active_query.pop();
    }

    pub fn active_newline(&mut self) {
        self.active_query.push('\n');
    }

    pub fn run_active_cell(&mut self) {
        self.ensure_backend();
        let sql = self.active_query.clone();
        if sql.trim().is_empty() {
            self.status = Some("empty query".into());
            return;
        }
        let Some(b) = &self.backend else {
            self.cells.push(Cell {
                query: sql,
                result: None,
                error: Some("no active connection".into()),
                duration_ms: 0,
                status: CellStatus::Error,
            });
            self.active_query.clear();
            return;
        };
        let start = Instant::now();
        let res = self.rt.block_on(b.run_query(&sql));
        let duration_ms = start.elapsed().as_millis();
        let cell = match res {
            Ok(qr) => {
                let n = qr.rows.len();
                self.status = Some(format!("ok: {n} row(s) in {duration_ms} ms"));
                Cell {
                    query: sql,
                    result: Some(qr),
                    error: None,
                    duration_ms,
                    status: CellStatus::Ok,
                }
            }
            Err(e) => {
                self.status = Some("query error".into());
                Cell {
                    query: sql,
                    result: None,
                    error: Some(format!("{e}")),
                    duration_ms,
                    status: CellStatus::Error,
                }
            }
        };
        self.cells.push(cell);
        self.active_query.clear();
        self.scroll_offset = 0;
    }

    pub fn new_cell(&mut self) {
        self.active_query.clear();
        self.scroll_offset = 0;
    }

    pub const fn scroll_up(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
    }

    pub const fn scroll_down(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn open_goto(&mut self) {
        self.ensure_backend();
        self.mode = Mode::Goto;
        self.goto_focus = GotoFocus::Connections;
        self.goto_conn_idx = self.selected;
        self.goto_table_idx = 0;
    }

    pub fn goto_next(&mut self) {
        match self.goto_focus {
            GotoFocus::Connections => {
                if !self.connections.connections.is_empty() {
                    self.goto_conn_idx =
                        (self.goto_conn_idx + 1) % self.connections.connections.len();
                }
            }
            GotoFocus::Tables => {
                if !self.tables.is_empty() {
                    self.goto_table_idx = (self.goto_table_idx + 1) % self.tables.len();
                }
            }
        }
    }

    pub fn goto_prev(&mut self) {
        match self.goto_focus {
            GotoFocus::Connections => {
                if !self.connections.connections.is_empty() {
                    if self.goto_conn_idx == 0 {
                        self.goto_conn_idx = self.connections.connections.len() - 1;
                    } else {
                        self.goto_conn_idx -= 1;
                    }
                }
            }
            GotoFocus::Tables => {
                if !self.tables.is_empty() {
                    if self.goto_table_idx == 0 {
                        self.goto_table_idx = self.tables.len() - 1;
                    } else {
                        self.goto_table_idx -= 1;
                    }
                }
            }
        }
    }

    pub const fn goto_toggle(&mut self) {
        self.goto_focus = match self.goto_focus {
            GotoFocus::Connections => GotoFocus::Tables,
            GotoFocus::Tables => GotoFocus::Connections,
        };
    }

    pub fn goto_commit(&mut self) {
        match self.goto_focus {
            GotoFocus::Connections => {
                if self.goto_conn_idx < self.connections.connections.len() {
                    self.selected = self.goto_conn_idx;
                    self.invalidate_backend();
                    self.ensure_backend();
                }
                self.mode = Mode::Normal;
            }
            GotoFocus::Tables => {
                if let Some(t) = self.tables.get(self.goto_table_idx) {
                    let snippet = format!("SELECT * FROM {} LIMIT 50", t.name);
                    if !self.active_query.is_empty() {
                        self.active_query.push('\n');
                    }
                    self.active_query.push_str(&snippet);
                }
                self.mode = Mode::Normal;
            }
        }
    }

    pub const fn close_goto(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn add_placeholder(&mut self) -> Result<()> {
        let n = self.connections.connections.len() + 1;
        let conn = Connection::new(
            format!("New SQLite {n}"),
            Engine::Sqlite,
            "sqlite::memory:".to_owned(),
        );
        self.connections.add(conn);
        self.selected = self.connections.connections.len() - 1;
        self.connections.save()?;
        self.invalidate_backend();
        self.status = Some("connection added".into());
        Ok(())
    }

    pub fn refresh_schema(&mut self) {
        self.ensure_backend();
        if let Some(b) = &self.backend
            && let Ok(tables) = self.rt.block_on(b.list_tables(None))
        {
            self.tables = tables;
        }
    }

    fn ensure_backend(&mut self) {
        let Some(conn) = self.current() else {
            self.backend = None;
            self.tables.clear();
            self.loaded_id = None;
            return;
        };
        if self.loaded_id.as_deref() == Some(conn.id.as_str()) && self.backend.is_some() {
            return;
        }
        let dsn = conn.dsn.clone();
        let id = conn.id.clone();
        let engine = conn.engine;
        let result: Result<Box<dyn Backend>> = match engine {
            Engine::Sqlite => self
                .rt
                .block_on(SqliteBackend::connect(if dsn.is_empty() {
                    "sqlite::memory:"
                } else {
                    &dsn
                }))
                .map(|b| Box::new(b) as Box<dyn Backend>),
            other => Err(anyhow::anyhow!(
                "{} backend not implemented yet",
                other.label()
            )),
        };
        match result {
            Ok(b) => {
                self.backend = Some(b);
                self.loaded_id = Some(id);
                self.refresh_schema();
            }
            Err(e) => {
                self.backend = None;
                self.loaded_id = None;
                self.tables.clear();
                self.status = Some(format!("connect: {e}"));
            }
        }
    }

    fn invalidate_backend(&mut self) {
        self.backend = None;
        self.loaded_id = None;
        self.tables.clear();
    }

    pub fn current(&self) -> Option<&Connection> {
        self.connections.connections.get(self.selected)
    }
}
