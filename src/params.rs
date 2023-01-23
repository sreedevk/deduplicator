use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use globwalk::{GlobWalker, GlobWalkerBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Filetypes to deduplicate (default = all)
    #[arg(short, long)]
    pub types: Option<String>,
    /// Run Deduplicator on dir different from pwd
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub dir: Option<PathBuf>,
    /// Delete files interactively
    #[arg(long, short)]
    pub interactive: bool,
    /// Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T). [default = 0]
    #[arg(long, short)]
    pub minsize: Option<String>,
}

impl Params {
    pub fn get_minsize(&self) -> Option<u64> {
        match &self.minsize {
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

    pub fn get_glob_walker(&self) -> Result<GlobWalker> {
        let pattern: String = match self.types.as_ref() {
            Some(filetypes) => format!("**/*{{{filetypes}}}"),
            None => "**/*".to_string(),
        };
        // TODO: add params for maximum depth and following symlinks, then pass them to this builder
        GlobWalkerBuilder::from_patterns(self.get_directory()?, &[pattern])
            .build()
            .map_err(|e| anyhow!(e))
    }
}
