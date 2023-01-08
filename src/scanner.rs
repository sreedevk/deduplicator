use std::{fs, path::PathBuf};

use anyhow::Result;
use fxhash::hash32 as hasher;
use glob::glob;
use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    database::{self, File},
    params::Params,
};

pub fn duplicates(app_opts: &Params, connection: &sqlite::Connection) -> Result<Vec<File>> {
    let scan_results = scan(app_opts, connection)?;
    let base_path = app_opts.get_directory()?;

    index_files(scan_results, connection)?;
    database::duplicate_hashes(connection, &base_path)
}

fn get_glob_patterns(opts: &Params, directory: &str) -> Vec<PathBuf> {
    opts.types
        .clone()
        .unwrap_or_else(|| String::from("*"))
        .split(',')
        .map(|filetype| format!("*.{}", filetype))
        .map(|filetype| {
            vec![directory.to_owned(), String::from("**"), filetype]
                .iter()
                .collect()
        })
        .collect()
}

fn is_indexed_file(path: impl Into<String>, indexed: &[File]) -> bool {
    indexed
        .iter()
        .map(|file| file.path.clone())
        .contains(&path.into())
}

fn scan(app_opts: &Params, connection: &sqlite::Connection) -> Result<Vec<String>> {
    let directory = app_opts.get_directory()?;
    let glob_patterns: Vec<PathBuf> = get_glob_patterns(app_opts, &directory);
    let indexed_paths = database::indexed_paths(connection)?;
    let files: Vec<String> = glob_patterns
        .into_par_iter()
        .filter_map(|glob_pattern| glob(glob_pattern.as_os_str().to_str()?).ok())
        .flat_map(|file_vec| {
            file_vec
                .filter_map(|x| Some(x.ok()?.as_os_str().to_str()?.to_string()))
                .filter(|fpath| !is_indexed_file(fpath, &indexed_paths))
                .filter(|glob_result| {
                    fs::metadata(glob_result)
                        .map(|f| f.is_file())
                        .unwrap_or(false)
                })
                .collect::<Vec<String>>()
        })
        .collect();

    Ok(files)
}

fn index_files(files: Vec<String>, connection: &sqlite::Connection) -> Result<()> {
    let hashed: Vec<File> = files
        .into_par_iter()
        .filter_map(|file| {
            let hash = hash_file(&file).ok()?;
            Some(database::File { path: file, hash })
        })
        .collect();

    hashed
        .into_iter()
        .try_for_each(|file| database::put(&file, connection))
}

pub fn hash_file(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    let hash = hasher(&*file).to_string();

    Ok(hash)
}
