#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
}

impl App {
    pub const fn new() -> Self {
        Self { should_quit: false }
    }
}
