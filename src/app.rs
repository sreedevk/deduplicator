use std::sync::mpsc::channel;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use threadpool::ThreadPool;

use crate::params::Params;
use crate::server::{Message, Server};
use crate::tui::Tui;

pub struct App {
    tpool: ThreadPool,
    server: Arc<Server>,
    app_opts: Arc<Params>,
}

impl App {
    pub fn new(app_opts: Arc<Params>) -> Self {
        Self {
            tpool: ThreadPool::new(8),
            server: Arc::new(Server::new().expect("server init failed")),
            app_opts,
        }
    }

    pub fn start(&self) -> Result<()> {
        let (server_tx, server_rx) = channel::<Message>();
        let (app_tx, app_rx) = channel::<Message>();

        let server_ptr = self.server.clone();
        let tui_server_ptr = self.server.clone();
        let mut ui = Tui::new(app_tx.clone(), tui_server_ptr);
        let root_dir = self
            .app_opts
            .get_directory()?
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("path to str conv failed"))?
            .into_boxed_str();

        self.tpool.execute(move || {
            server_ptr.start(server_rx).expect("server init failed");
        });

        self.tpool.execute(move || {
            ui.start().expect("ui init failed");
        });

        self.tpool.execute(move || loop {
            match app_rx.try_recv() {
                Ok(Message::Exit) => {
                    server_tx
                        .send(Message::Exit)
                        .expect("message passing to app from ui failed.");
                    break;
                }
                Ok(Message::AddScanDirectory(dir)) => {
                    server_tx
                        .send(Message::AddScanDirectory(dir))
                        .expect("message passing to server failed.");
                }
                _ => continue,
            }
        });

        app_tx.send(Message::AddScanDirectory(root_dir))?;

        self.tpool.join();

        Ok(())
    }
}
