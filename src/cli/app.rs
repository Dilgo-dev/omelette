use anyhow::Result;

use crate::connections::ConnectionStore;

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    #[allow(dead_code)] // consumed by the connections list UI in #5
    pub connections: ConnectionStore,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            connections: ConnectionStore::load()?,
        })
    }
}
