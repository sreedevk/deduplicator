use anyhow::Result;
use fxhash::hash64 as hasher;
use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::hash::Hasher;
use std::{fs, path::PathBuf};
use memmap2::Mmap;
use dashmap::DashMap;

use crate::{file_manager::File, params::Params};

pub fn duplicates(app_opts: &Params) -> Result<Vec<File>> {
    let scan_results = scan(app_opts)?;
    let index_store = index_files(scan_results)?;

    let duplicate_files = index_store
        .into_par_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(_, files)| files )
        .flatten()
        .collect::<Vec<File>>();

    Ok(duplicate_files)
}

fn scan(app_opts: &Params) -> Result<Vec<String>> {
    let glob_patterns: Vec<PathBuf> = app_opts.get_glob_patterns();
    let files: Vec<String> = glob_patterns
        .par_iter()
        .progress_with_style(
            ProgressStyle::with_template(
                "{spinner:.green} [scanning files] [{wide_bar:.cyan/blue}] {pos}/{len} files",
            )
            .unwrap(),
        )
        .filter_map(|glob_pattern| glob(glob_pattern.as_os_str().to_str()?).ok())
        .flat_map(|file_vec| {
            file_vec
                .filter_map(|x| Some(x.ok()?.as_os_str().to_str()?.to_string()))
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

fn index_files(files: Vec<String>) -> Result<DashMap<String, Vec<File>>> {
    let store: DashMap<String, Vec<File>> = DashMap::new();
    files
        .into_par_iter()
        .progress_with_style(
            ProgressStyle::with_template(
                "{spinner:.green} [indexing files] [{wide_bar:.cyan/blue}] {pos}/{len} files",
            )?,
        )
        .for_each(|file| {
            let hash = hash_file(&file).unwrap_or_default();
            let fobj = File { path: file, hash: hash.clone() };
            store
                .entry(hash)
                .and_modify(|fileset| fileset.push(fobj.clone()) )
                .or_insert_with(|| vec![fobj]);
        });

        Ok(store)
}

pub fn incremental_hashing(filepath: &str) -> Result<String> {
    let file = fs::File::open(filepath)?;
    let fmap = unsafe { Mmap::map(&file)? };
    let mut inchasher = fxhash::FxHasher::default();

    fmap
        .chunks(1_000)
        .for_each(|kilo| { inchasher.write(kilo) });

    Ok(format!("{}", inchasher.finish()))
}

pub fn standard_hashing(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    Ok(hasher(&*file).to_string())
}

pub fn hash_file(filepath: &str) -> Result<String> {
    let filemeta = fs::metadata(filepath)?;

    match filemeta.len() < 1_000_000 {
        true => standard_hashing(filepath),
        false => incremental_hashing(filepath)
    }
}
