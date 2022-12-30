use crate::database;
use crate::{cli::App, database::File};
use anyhow::Result;
use glob::glob;
use itertools::Itertools;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

pub fn duplicates(app_opts: App, connection: &sqlite::Connection) -> Result<Vec<File>> {
    let scan_results = scan(&app_opts, connection)?;
    let base_path = get_directory(&app_opts)?;

    index_files(scan_results, connection);
    database::duplicate_hashes(connection, &base_path)
}

fn get_directory(opts: &App) -> Result<String> {
    let dir_string: String = opts
        .dir
        .clone()
        .unwrap_or(std::env::current_dir()?)
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    let dir_pathbuf = PathBuf::from(&dir_string);
    let dir = fs::canonicalize(&dir_pathbuf)?
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    Ok(dir)
}

fn get_glob_patterns(opts: &App, directory: &String) -> Vec<PathBuf> {
    opts.filetypes
        .clone()
        .unwrap_or(String::from("*"))
        .split(",")
        .map(|filetype| format!("*.{}", filetype))
        .map(|filetype| {
            vec![directory.clone(), String::from("**"), filetype]
                .iter()
                .collect()
        })
        .collect()
}

fn is_indexed_file(path: &String, indexed: &Vec<File>) -> bool {
    indexed
        .into_iter()
        .map(|file| file.path.clone())
        .contains(path)
}

fn scan(app_opts: &App, connection: &sqlite::Connection) -> Result<Vec<String>> {
    let directory = get_directory(app_opts)?;
    let glob_patterns: Vec<PathBuf> = get_glob_patterns(&app_opts, &directory);
    let indexed_paths = database::indexed_paths(connection)?;
    let files: Vec<String> = glob_patterns
        .into_par_iter()
        .map(|glob_pattern| glob(&glob_pattern.as_os_str().to_str().unwrap()))
        .map(|glob_result| glob_result.unwrap())
        .flat_map(|file_vec| {
            file_vec
                .map(|x| x.unwrap().as_os_str().to_str().unwrap().to_string())
                .filter(|fpath| !is_indexed_file(fpath, &indexed_paths))
                .filter(|glob_result| fs::metadata(glob_result).unwrap().is_file())
                .collect::<Vec<String>>()
        })
        .collect();

    Ok(files)
}

fn index_files(files: Vec<String>, connection: &sqlite::Connection) {
    let hashed: Vec<File> = files
        .into_par_iter()
        .map(|file| {
            let hash = hash_file(&file).unwrap();
            database::File { path: file, hash }
        })
        .collect();

    hashed.into_iter().for_each(|file| {
        database::put(&file, connection).unwrap();
    });
}

pub fn hash_file(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    let hash = sha256::digest(&*file);

    Ok(hash)
}
