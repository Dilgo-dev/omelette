use anyhow::Result;

use crate::connections::{Connection, ConnectionStore};
use omelette::core::engine::Engine;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    ConfirmDelete,
    Rename,
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub connections: ConnectionStore,
    pub selected: usize,
    pub mode: Mode,
    pub rename_buffer: String,
    pub status: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            connections: ConnectionStore::load()?,
            selected: 0,
            mode: Mode::Normal,
            rename_buffer: String::new(),
            status: None,
        })
    }

    pub fn select_next(&mut self) {
        if self.connections.connections.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1) % self.connections.connections.len();
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
    }

    pub fn add_placeholder(&mut self) -> Result<()> {
        let n = self.connections.connections.len() + 1;
        let conn = Connection::new(format!("New SQLite {n}"), Engine::Sqlite, String::new());
        self.connections.add(conn);
        self.selected = self.connections.connections.len() - 1;
        self.connections.save()?;
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

    pub fn current(&self) -> Option<&Connection> {
        self.connections.connections.get(self.selected)
    }
}
