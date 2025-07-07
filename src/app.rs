use anyhow::Result;
use threadpool::ThreadPool;

use crate::pipeline::Server;
use crate::tui::Tui;

pub struct App {
    tpool: ThreadPool,
    server: Server,
    ui: Tui,
}

impl App {
    pub fn new() -> Self {
        Self {
            tpool: ThreadPool::new(8),
            server: Server::new().expect("server init failed"),
            ui: Tui {},
        }
    }

    pub fn start() -> Result<()> {
        Ok(())
    }
}
