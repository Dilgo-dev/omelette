use anyhow::Result;
use tokio::runtime::{Builder, Runtime};

use crate::connections::{Connection, ConnectionStore};
use omelette::core::backend::{Backend, TableName};
use omelette::core::engine::Engine;
use omelette::core::result::QueryResult;
use omelette::core::sqlite::SqliteBackend;

const PREVIEW_LIMIT: u32 = 50;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    #[default]
    Connections,
    Schema,
    Preview,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    ConfirmDelete,
    Rename,
    EditingQuery,
}

pub struct App {
    pub should_quit: bool,
    pub connections: ConnectionStore,
    pub selected: usize,
    pub focus: Focus,
    pub mode: Mode,
    pub rename_buffer: String,
    pub status: Option<String>,

    pub tables: Vec<TableName>,
    pub selected_table: usize,
    pub loaded_id: Option<String>,
    pub preview: Option<QueryResult>,
    pub preview_row_offset: usize,
    pub preview_col_offset: usize,
    pub previewed_table: Option<String>,

    pub query_buffer: String,
    pub query_result: Option<QueryResult>,
    pub query_error: Option<String>,
    backend: Option<Box<dyn Backend>>,
    rt: Runtime,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("should_quit", &self.should_quit)
            .field("selected", &self.selected)
            .field("focus", &self.focus)
            .field("mode", &self.mode)
            .field("loaded_id", &self.loaded_id)
            .field("tables", &self.tables.len())
            .finish_non_exhaustive()
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let rt = Builder::new_current_thread().enable_all().build()?;
        Ok(Self {
            should_quit: false,
            connections: ConnectionStore::load()?,
            selected: 0,
            focus: Focus::Connections,
            mode: Mode::Normal,
            rename_buffer: String::new(),
            status: None,
            tables: Vec::new(),
            selected_table: 0,
            loaded_id: None,
            preview: None,
            preview_row_offset: 0,
            preview_col_offset: 0,
            previewed_table: None,
            query_buffer: String::new(),
            query_result: None,
            query_error: None,
            backend: None,
            rt,
        })
    }

    pub fn enter_query_mode(&mut self) {
        self.ensure_backend();
        self.mode = Mode::EditingQuery;
        self.query_error = None;
    }

    pub fn query_push(&mut self, c: char) {
        self.query_buffer.push(c);
    }

    pub fn query_pop(&mut self) {
        self.query_buffer.pop();
    }

    pub fn query_newline(&mut self) {
        self.query_buffer.push('\n');
    }

    pub fn run_query(&mut self) {
        let Some(b) = &self.backend else {
            self.query_error = Some("no active connection".into());
            return;
        };
        let sql = self.query_buffer.clone();
        if sql.trim().is_empty() {
            self.query_error = Some("empty query".into());
            return;
        }
        match self.rt.block_on(b.run_query(&sql)) {
            Ok(qr) => {
                let n = qr.rows.len();
                self.query_result = Some(qr);
                self.query_error = None;
                self.status = Some(format!("query ok: {n} row(s)"));
            }
            Err(e) => {
                self.query_error = Some(format!("{e}"));
                self.status = Some("query error".into());
            }
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Connections => Focus::Schema,
            Focus::Schema => Focus::Preview,
            Focus::Preview => Focus::Connections,
        };
        if self.focus == Focus::Schema {
            self.ensure_backend();
        }
    }

    pub fn select_next(&mut self) {
        if self.connections.connections.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1) % self.connections.connections.len();
        self.invalidate_backend();
    }

    pub fn select_prev(&mut self) {
        if self.connections.connections.is_empty() {
            self.selected = 0;
            return;
        }
        if self.selected == 0 {
            self.selected = self.connections.connections.len() - 1;
        } else {
            self.selected -= 1;
        }
        self.invalidate_backend();
    }

    pub fn select_next_table(&mut self) {
        if self.tables.is_empty() {
            self.selected_table = 0;
            return;
        }
        self.selected_table = (self.selected_table + 1) % self.tables.len();
        self.load_preview();
    }

    pub fn select_prev_table(&mut self) {
        if self.tables.is_empty() {
            self.selected_table = 0;
            return;
        }
        if self.selected_table == 0 {
            self.selected_table = self.tables.len() - 1;
        } else {
            self.selected_table -= 1;
        }
        self.load_preview();
    }

    pub fn scroll_preview_down(&mut self) {
        if let Some(p) = &self.preview {
            if self.preview_row_offset + 1 < p.rows.len() {
                self.preview_row_offset += 1;
            }
        }
    }

    pub const fn scroll_preview_up(&mut self) {
        self.preview_row_offset = self.preview_row_offset.saturating_sub(1);
    }

    pub fn scroll_preview_right(&mut self) {
        if let Some(p) = &self.preview {
            if self.preview_col_offset + 1 < p.columns.len() {
                self.preview_col_offset += 1;
            }
        }
    }

    pub const fn scroll_preview_left(&mut self) {
        self.preview_col_offset = self.preview_col_offset.saturating_sub(1);
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

    pub fn delete_selected(&mut self) -> Result<()> {
        let Some(conn) = self.current() else {
            return Ok(());
        };
        let id = conn.id.clone();
        self.connections.remove(&id);
        if self.selected >= self.connections.connections.len()
            && !self.connections.connections.is_empty()
        {
            self.selected = self.connections.connections.len() - 1;
        }
        self.connections.save()?;
        self.invalidate_backend();
        self.status = Some("connection deleted".into());
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn start_rename(&mut self) {
        if let Some(c) = self.current() {
            self.rename_buffer = c.label.clone();
            self.mode = Mode::Rename;
        }
    }

    pub fn commit_rename(&mut self) -> Result<()> {
        if let Some(c) = self.current() {
            let id = c.id.clone();
            let new_label = self.rename_buffer.clone();
            self.connections.update(&id, |c| c.label = new_label);
            self.connections.save()?;
            self.status = Some("renamed".into());
        }
        self.mode = Mode::Normal;
        self.rename_buffer.clear();
        Ok(())
    }

    pub fn cancel_mode(&mut self) {
        self.mode = Mode::Normal;
        self.rename_buffer.clear();
    }

    pub const fn exit_query_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn refresh_schema(&mut self) {
        self.ensure_backend();
        if let Some(b) = &self.backend {
            match self.rt.block_on(b.list_tables(None)) {
                Ok(tables) => {
                    self.tables = tables;
                    if self.selected_table >= self.tables.len() {
                        self.selected_table = 0;
                    }
                    self.status = Some(format!("loaded {} table(s)", self.tables.len()));
                }
                Err(e) => self.status = Some(format!("list_tables: {e}")),
            }
        }
        self.load_preview();
    }

    pub fn load_preview(&mut self) {
        let Some(table) = self.tables.get(self.selected_table).cloned() else {
            self.preview = None;
            self.previewed_table = None;
            return;
        };
        if self.previewed_table.as_deref() == Some(table.name.as_str()) {
            return;
        }
        let Some(b) = &self.backend else {
            return;
        };
        match self.rt.block_on(b.preview_table(&table, PREVIEW_LIMIT)) {
            Ok(qr) => {
                let rows = qr.rows.len();
                self.preview = Some(qr);
                self.previewed_table = Some(table.name.clone());
                self.preview_row_offset = 0;
                self.preview_col_offset = 0;
                self.status = Some(format!("preview {}: {rows} row(s)", table.name));
            }
            Err(e) => {
                self.preview = None;
                self.previewed_table = None;
                self.status = Some(format!("preview: {e}"));
            }
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
        self.selected_table = 0;
        self.preview = None;
        self.previewed_table = None;
        self.preview_row_offset = 0;
        self.preview_col_offset = 0;
    }

    pub fn current(&self) -> Option<&Connection> {
        self.connections.connections.get(self.selected)
    }
}
