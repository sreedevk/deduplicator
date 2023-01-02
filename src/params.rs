use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Filetypes to deduplicate (default = all)
    #[arg(short, long)]
    pub types: Option<String>,
    /// Run Deduplicator on dir different from pwd
    #[arg(long)]
    pub dir: Option<PathBuf>,
    /// Don't use cache for indexing files (default = true)
    #[arg(long, short)]
    pub nocache: bool,
}

impl Params {
    pub fn get_directory(&self) -> Result<String> {
        let dir_string: String = self
            .dir
            .clone()
            .unwrap_or(std::env::current_dir()?)
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        let dir_pathbuf = PathBuf::from(&dir_string);
        let dir = fs::canonicalize(&dir_pathbuf)?
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        Ok(dir)
    }
}
