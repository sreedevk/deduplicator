use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, ValueHint};

#[derive(Parser, Debug, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Exclude Filetypes [default = none]
    #[arg(short = 'T', long)]
    pub exclude_types: Option<String>,
    /// Filetypes to deduplicate [default = all]
    #[arg(short, long)]
    pub types: Option<String>,
    /// Run Deduplicator on dir different from pwd (e.g., ~/Pictures )
    #[arg(value_hint = ValueHint::DirPath, value_name = "scan_dir_path")]
    pub dir: Option<PathBuf>,
    /// Delete files interactively
    #[arg(long, short)]
    pub interactive: bool,
    /// Keep one file per duplicate group by this rule and remove the rest
    #[arg(long, conflicts_with = "interactive")]
    pub keep: Option<crate::resolver::KeepStrategy>,
    /// Actually delete the duplicates (without this, --keep only previews)
    #[arg(long, visible_alias = "yes", requires = "keep")]
    pub force: bool,
    /// Minimum filesize of duplicates to scan (e.g., 100B/1K/2M/3G/4T).
    #[arg(long, short = 'm', default_value = "1b")]
    pub min_size: Option<String>,
    /// Max Depth to scan while looking for duplicates
    #[arg(long, short = 'D')]
    pub max_depth: Option<usize>,
    /// Min Depth to scan while looking for duplicates
    #[arg(long, short = 'd')]
    pub min_depth: Option<usize>,
    /// Follow links while scanning directories
    #[arg(long, short)]
    pub follow_links: bool,
    /// Guarantees that two files are duplicate (performs a full hash)
    #[arg(long, short = 's', default_value = "false")]
    pub strict: bool,
    /// Show Progress spinners & metrics
    #[arg(long, short = 'p', default_value = "false")]
    pub progress: bool,
    /// Disable the on-disk hash cache
    #[arg(long)]
    pub no_cache: bool,
    /// Use a specific cache file instead of the default location
    #[arg(long, value_name = "PATH")]
    pub cache_file: Option<PathBuf>,
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
}
