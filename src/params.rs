use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use globwalk::{GlobWalker, GlobWalkerBuilder};

#[derive(Parser, Debug)]
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
    #[arg(long, default_value = "1b")]
    pub min_size: Option<String>,
    /// Max Depth to scan while looking for duplicates
    #[arg(long, short = 'd')]
    pub max_depth: Option<usize>,
    /// Min Depth to scan while looking for duplicates
    #[arg(long)]
    pub min_depth: Option<usize>,
    /// Follow links while scanning directories
    #[arg(long)]
    pub follow_links: bool,
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

    fn add_glob_min_depth(&self, builder: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        match self.min_depth {
            Some(mindepth) => Ok(builder.min_depth(mindepth)),
            None => Ok(builder),
        }
    }

    fn add_glob_max_depth(&self, builder: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        match self.max_depth {
            Some(maxdepth) => Ok(builder.max_depth(maxdepth)),
            None => Ok(builder),
        }
    }

    fn add_glob_follow_links(&self, builder: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        match self.follow_links {
            true => Ok(builder.follow_links(true)),
            false => Ok(builder.follow_links(false)),
        }
    }

    pub fn get_glob_walker(&self) -> Result<GlobWalker> {
        let pattern: String = match self.types.as_ref() {
            Some(filetypes) => format!("**/*{{{filetypes}}}"),
            None => "**/*".to_string(),
        };

        let glob_walker_builder = self
            .add_glob_min_depth(GlobWalkerBuilder::from_patterns(
                self.get_directory()?,
                &[pattern],
            ))
            .and_then(|builder| self.add_glob_max_depth(builder))
            .and_then(|builder| self.add_glob_follow_links(builder))?;

        glob_walker_builder.build().map_err(|e| anyhow!(e))
    }
}
