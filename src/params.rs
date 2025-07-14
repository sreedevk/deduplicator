use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, ValueHint};
use std::collections::HashSet;

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

    pub fn types_intersection(itypes: &str, xtypes: &str) -> String {
        let iset = itypes
            .split(",")
            .map(String::from)
            .collect::<HashSet<String>>();
        let xset = xtypes
            .split(",")
            .map(String::from)
            .collect::<HashSet<String>>();

        iset.difference(&xset)
            .cloned()
            .collect::<Vec<String>>()
            .join(",")
    }

    pub fn get_types(&self) -> Option<String> {
        match &self.types {
            Some(itypes) => match &self.exclude_types {
                Some(xtypes) => Some(Self::types_intersection(itypes, xtypes)),
                None => Some(itypes.to_string()),
            },
            None => self.exclude_types.as_ref().map(|xtypes| xtypes.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixing_include_and_exclude_types_works_as_expected() {
        let params = Params {
            types: Some(String::from("js,xml,ts,pdf,tiff")),
            exclude_types: Some(String::from("js,ts,xml")),
            ..Default::default()
        };

        assert!(params
            .get_types()
            .is_some_and(|x| x == "pdf,tiff" || x == "tiff,pdf"))
    }
}
