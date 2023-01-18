use anyhow::Result;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub size: Option<u64>,
    pub hash: Option<String>,
}

pub fn delete_files(files: Vec<File>) -> Result<()> {
    files.into_iter().for_each(|file| {
        match std::fs::remove_file(file.path.clone()) {
            Ok(_) => println!("{}: {}", "DELETED".green(), file.path),
            Err(_) => println!("{}: {}", "FAILED".red(), file.path)
        }
    });

    Ok(())
}
