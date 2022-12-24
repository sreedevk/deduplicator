use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct App {
    /// Filetypes to deduplicate (default = all)
    #[arg(short, long)]
    pub filetypes: Option<String>,
    /// List duplcates without deleting (default = true)
    #[arg(short, long)]
    pub dry: bool,
    /// Run Deduplicator on dir different from pwd
    #[arg(long)]
    pub dir: Option<PathBuf>
}
