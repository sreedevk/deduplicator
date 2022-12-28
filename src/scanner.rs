use std::path::PathBuf;

use crate::{cli::App, database::File};
use crate::database;
use anyhow::Result;
use glob::glob;
use std::fs;
use rayon::prelude::*;

pub fn duplicates(app_opts: App, connection: &sqlite::Connection) -> Result<Vec<File>> {
    index_files(scan(app_opts)?, connection);
    database::duplicate_hashes(connection)
}

fn scan(app_opts: App) -> Result<Vec<String>> {
    let directory: String = app_opts
        .dir
        .unwrap_or(std::env::current_dir()?)
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    let glob_patterns: Vec<PathBuf> = app_opts
        .filetypes
        .unwrap_or(String::from("*"))
        .split(",")
        .map(|filetype| format!("*.{}", filetype))
        .map(|filetype| {
            vec![directory.clone(), String::from("**"), filetype]
                .iter()
                .collect()
        })
        .collect();

    let files: Vec<String> = glob_patterns
        .into_par_iter()
        .map(|glob_pattern| glob(&glob_pattern.as_os_str().to_str().unwrap()))
        .map(|glob_result| glob_result.unwrap())
        .flat_map(|file_vec| {
            file_vec
                .map(|x| x.unwrap().as_os_str().to_str().unwrap().to_string())
                .filter(|glob_result| fs::metadata(glob_result).unwrap().is_file() )
                .collect::<Vec<String>>()
        })
        .collect();

    Ok(files)
}

fn index_files(files: Vec<String>, connection: &sqlite::Connection) {
    files
        .into_iter()
        .map(|file| {
            let hash = hash_file(&file).unwrap();
            database::File { path: file, hash }
        })
        .for_each(|file| {
            database::put(&file, connection).unwrap();
        });
}

pub fn hash_file(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    let hash = sha256::digest(&*file);

    Ok(hash)
}
