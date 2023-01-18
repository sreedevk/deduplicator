use anyhow::Result;
use dashmap::DashMap;
use fxhash::hash64 as hasher;
use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use memmap2::Mmap;
use rayon::prelude::*;
use std::hash::Hasher;
use std::{fs, path::PathBuf};

use crate::{file_manager::File, params::Params};

#[derive(Clone, Copy)]
enum IndexCritera {
    Size,
    Hash,
}

pub fn duplicates(app_opts: &Params) -> Result<DashMap<String, Vec<File>>> {
    let scan_results = scan(app_opts)?;
    let size_index_store = index_files(scan_results, IndexCritera::Size)?;

    let sizewize_duplicate_files = size_index_store
        .into_par_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(_, files)| files)
        .flatten()
        .collect::<Vec<File>>();

    if sizewize_duplicate_files.len() > 1 {
        let size_wise_duplicate_paths = sizewize_duplicate_files
            .into_par_iter()
            .map(|file| file.path)
            .collect::<Vec<String>>();

        let hash_index_store = index_files(size_wise_duplicate_paths, IndexCritera::Hash)?;
        let duplicate_files = hash_index_store
            .into_par_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        Ok(duplicate_files)
    } else {
        Ok(DashMap::new())
    }
}

fn scan(app_opts: &Params) -> Result<Vec<String>> {
    let glob_patterns: Vec<PathBuf> = app_opts.get_glob_patterns();
    let files: Vec<String> = glob_patterns
        .par_iter()
        .progress_with_style(ProgressStyle::with_template(
            "{spinner:.green} [scanning files] [{wide_bar:.cyan/blue}] {pos}/{len} files",
        )?)
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

fn process_file_size_index(fpath: String) -> Result<File> {
    Ok(File {
        path: fpath.clone(),
        size: Some(fs::metadata(fpath)?.len()),
        hash: None,
    })
}

fn process_file_hash_index(fpath: String) -> Result<File> {
    Ok(File {
        path: fpath.clone(),
        size: None,
        hash: Some(hash_file(&fpath).unwrap_or_default()),
    })
}

fn process_file_index(
    fpath: String,
    store: &DashMap<String, Vec<File>>,
    index_criteria: IndexCritera,
) {
    match index_criteria {
        IndexCritera::Size => {
            let processed_file = process_file_size_index(fpath).unwrap();
            store
                .entry(processed_file.size.unwrap_or_default().to_string())
                .and_modify(|fileset| fileset.push(processed_file.clone()))
                .or_insert_with(|| vec![processed_file]);
        }
        IndexCritera::Hash => {
            let processed_file = process_file_hash_index(fpath).unwrap();
            let indexhash = processed_file.clone().hash.unwrap_or_default();

            store
                .entry(indexhash)
                .and_modify(|fileset| fileset.push(processed_file.clone()))
                .or_insert_with(|| vec![processed_file]);
        }
    }
}

fn index_files(
    files: Vec<String>,
    index_criteria: IndexCritera,
) -> Result<DashMap<String, Vec<File>>> {
    let store: DashMap<String, Vec<File>> = DashMap::new();
    files
        .into_par_iter()
        .progress_with_style(ProgressStyle::with_template(
            "{spinner:.green} [indexing files] [{wide_bar:.cyan/blue}] {pos}/{len} files",
        )?)
        .for_each(|file| process_file_index(file, &store, index_criteria));

    Ok(store)
}

pub fn incremental_hashing(filepath: &str) -> Result<String> {
    let file = fs::File::open(filepath)?;
    let fmap = unsafe { Mmap::map(&file)? };
    let mut inchasher = fxhash::FxHasher::default();

    fmap.chunks(1_000_000)
        .for_each(|mega| inchasher.write(mega));

    Ok(format!("{}", inchasher.finish()))
}

pub fn standard_hashing(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    Ok(hasher(&*file).to_string())
}

pub fn hash_file(filepath: &str) -> Result<String> {
    let filemeta = fs::metadata(filepath)?;

    // NOTE: USE INCREMENTAL HASHING ONLY FOR FILES > 100MB
    match filemeta.len() < 100_000_000 {
        true => standard_hashing(filepath),
        false => incremental_hashing(filepath),
    }
}
