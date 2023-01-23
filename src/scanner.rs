use crate::{file_manager::File, filters, params::Params};
use anyhow::Result;
use dashmap::DashMap;
use fxhash::hash64 as hasher;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use memmap2::Mmap;
use rayon::prelude::*;
use std::hash::Hasher;
use std::{fs, path::PathBuf};

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
        let hash_index_store = index_files(sizewize_duplicate_files, IndexCritera::Hash)?;
        let duplicate_files = hash_index_store
            .into_par_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        Ok(duplicate_files)
    } else {
        Ok(DashMap::new())
    }
}

fn scan(app_opts: &Params) -> Result<Vec<File>> {
    let glob_patterns = app_opts.get_glob_patterns().display().to_string();
    let glob_iter = globwalk::glob(glob_patterns)?;
    let files = glob_iter
        .filter_map(Result::ok)
        .map(|file| file.into_path())
        .filter(|fpath| fpath.is_file())
        .collect::<Vec<PathBuf>>()
        .into_par_iter()
        .progress_with_style(ProgressStyle::with_template(
            "{spinner:.green} [processing scan results] [{wide_bar:.cyan/blue}] {pos}/{len} files",
        )?)
        .map(|fpath| fpath.display().to_string())
        .map(|fpath| File {
            path: fpath.clone(),
            hash: None,
            size: Some(fs::metadata(fpath).unwrap().len()),
        })
        .filter(|file| filters::is_file_gt_minsize(app_opts, file))
        .collect();

    Ok(files)
}

fn process_file_index(
    mut file: File,
    store: &DashMap<String, Vec<File>>,
    index_criteria: IndexCritera,
) {
    match index_criteria {
        IndexCritera::Size => {
            store
                .entry(file.size.unwrap_or_default().to_string())
                .and_modify(|fileset| fileset.push(file.clone()))
                .or_insert_with(|| vec![file]);
        }
        IndexCritera::Hash => {
            file.hash = Some(hash_file(&file.path).unwrap_or_default());
            store
                .entry(file.clone().hash.unwrap())
                .and_modify(|fileset| fileset.push(file.clone()))
                .or_insert_with(|| vec![file]);
        }
    }
}

fn index_files(
    files: Vec<File>,
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

fn incremental_hashing(filepath: &str) -> Result<String> {
    let file = fs::File::open(filepath)?;
    let fmap = unsafe { Mmap::map(&file)? };
    let mut inchasher = fxhash::FxHasher::default();

    fmap.chunks(1_000_000)
        .for_each(|mega| inchasher.write(mega));

    Ok(format!("{}", inchasher.finish()))
}

fn standard_hashing(filepath: &str) -> Result<String> {
    let file = fs::read(filepath)?;
    Ok(hasher(&*file).to_string())
}

fn hash_file(filepath: &str) -> Result<String> {
    let filemeta = fs::metadata(filepath)?;

    // NOTE: USE INCREMENTAL HASHING ONLY FOR FILES > 100MB
    match filemeta.len() < 100_000_000 {
        true => standard_hashing(filepath),
        false => incremental_hashing(filepath),
    }
}
