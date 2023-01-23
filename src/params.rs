use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use std::{fs, path::PathBuf};

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

    pub fn get_directory(&self) -> Result<String> {
        let dir_pathbuf: PathBuf = self
            .dir
            .as_ref()
            .unwrap_or(&std::env::current_dir()?)
            .as_os_str()
            .into();

        let dir = fs::canonicalize(dir_pathbuf)?
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid directory"))?
            .to_string();

        Ok(dir)
    }

    pub fn get_glob_patterns(&self) -> PathBuf {
        match self.types.as_ref() {
            Some(filetypes) => vec![
                self.get_directory().unwrap(),
                String::from("**"),
                format!("*.{{{filetypes}}}"),
            ]
            .iter()
            .collect::<PathBuf>(),
            None => vec![self.get_directory().unwrap().as_str(), "**", "*"]
                .iter()
                .collect::<PathBuf>(),
        }
    }
}
