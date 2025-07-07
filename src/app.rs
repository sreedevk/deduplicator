use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use anyhow::Result;
use threadpool::ThreadPool;

use crate::pipeline::{Message, Server};
use crate::tui::Tui;

pub struct App {
    tpool: ThreadPool,
    server: Arc<Server>,
}

impl App {
    pub fn new() -> Self {
        Self {
            tpool: ThreadPool::new(8),
            server: Arc::new(Server::new().expect("server init failed")),
        }
    }

    pub fn start(&self) -> Result<()> {
        let (server_tx, server_rx) = channel::<Message>();
        let (app_tx, app_rx) = channel::<Message>();

        let server_ptr = self.server.clone();
        let ui = Tui::new(app_tx);

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
                        .expect("message passing to app from ui failed");
                    break;
                }
                _ => continue,
            }
        });

        self.tpool.join();

        Ok(())
    }
}
