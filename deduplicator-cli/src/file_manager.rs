use anyhow::Result;
use colored::Colorize;
use deduplicator_core::fileinfo::FileInfo;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub size: Option<u64>,
    pub hash: Option<String>,
}

pub fn delete_files(files: Vec<FileInfo>) -> Result<()> {
    files.into_iter().for_each(|file| {
        match std::fs::remove_file(file.path.clone()) {
            Ok(_) => println!("{}: {}", "DELETED".green(), file.path.display()),
            Err(_) => println!("{}: {}", "FAILED".red(), file.path.display())
        }
    });

    Ok(())
}
