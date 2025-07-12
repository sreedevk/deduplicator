use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, ValueHint};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Filetypes to deduplicate [default = all]
    #[arg(short, long)]
    pub types: Option<String>,
    /// Run Deduplicator on dir different from pwd (e.g., ~/Pictures )
    #[arg(value_hint = ValueHint::DirPath, value_name = "scan_dir_path")]
    pub dir: Option<PathBuf>,
    /// Delete files interactively
    #[arg(long, short)]
    pub interactive: bool,
    /// Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T).
    #[arg(long, short = 's', default_value = "1b")]
    pub min_size: Option<String>,
    /// Max Depth to scan while looking for duplicates
    #[arg(long, short = 'd')]
    pub max_depth: Option<usize>,
    /// Min Depth to scan while looking for duplicates
    #[arg(long)]
    pub min_depth: Option<usize>,
    /// Follow links while scanning directories
    #[arg(long, short)]
    pub follow_links: bool,
    /// print json output
    #[arg(long)]
    pub json: bool,
    /// Guarantees that two files are duplicate (performs a full hash)
    #[arg(long, short = 'f', default_value = "false")]
    pub strict: bool,
}

impl Params {
    pub fn get_min_size(&self) -> Option<u64> {
        match &self.min_size {
            Some(msize) => match msize.parse::<bytesize::ByteSize>() {
                Ok(units) => Some(units.0),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn get_directory(&self) -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;
        let dir_path = self.dir.as_ref().unwrap_or(&current_dir).as_path();
        let dir = fs::canonicalize(dir_path)?;
        Ok(dir)
    }

    pub fn get_types(&self) -> Option<String> {
        self.types.clone()
    }
}
