use crate::database::File;
use anyhow::Result;
use colored::Colorize;

pub fn delete_files(files: Vec<File>) -> Result<()> {
    files.into_iter().for_each(|file| {
        match std::fs::remove_file(file.path.clone()) {
            Ok(_) => println!("{}: {}", "DELETED".green(), file.path),
            Err(e) => println!("{}: {}", "FAILED".red(), file.path)
        }
    });

    Ok(())
}
