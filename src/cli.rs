use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct App {
    /// Filetypes to deduplicate (default = all)
    #[arg(short, long)]
    pub types: Option<String>,
    /// Delete files by algorithm (default = oldest) [options = oldest | newest]
    #[arg(short, long)]
    pub delete: Option<String>,
    /// Run Deduplicator on dir different from pwd
    #[arg(long)]
    pub dir: Option<PathBuf>
}
