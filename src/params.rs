use std::{fs, path::PathBuf};
use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Filetypes to deduplicate (default = all)
    #[arg(short, long)]
    pub types: Option<String>,
    /// Run Deduplicator on dir different from pwd
    #[arg(long)]
    pub dir: Option<PathBuf>,
    /// Don't use cache for indexing files (default = false)
    #[arg(long, short)]
    pub nocache: bool,
    /// Delete files interactively
    #[arg(long, short)]
    pub interactive: bool,
}

impl Params {
    pub fn get_directory(&self) -> Result<String> {
        let dir_pathbuf: PathBuf = self
            .dir
            .clone()
            .unwrap_or(std::env::current_dir()?)
            .as_os_str()
            .into();

        let dir = fs::canonicalize(dir_pathbuf)?
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid directory"))?
            .to_string();

        Ok(dir)
    }
}
