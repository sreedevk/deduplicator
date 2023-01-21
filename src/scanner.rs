use crate::{file_manager::File, filters, params::Params};
use anyhow::Result;
use dashmap::DashMap;
use fxhash::hash64 as hasher;
use glob::glob;
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

    if sizewize_duplicate_files.is_empty() {
        Ok(DashMap::new())
    } else {
        let hash_index_store = index_files(sizewize_duplicate_files, IndexCritera::Hash)?;
        let duplicate_files = hash_index_store
            .into_par_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        Ok(duplicate_files)
    }
}

fn scan(app_opts: &Params) -> Result<Vec<File>> {
    let glob_patterns: Vec<PathBuf> = app_opts.get_glob_patterns();
    let files: Vec<File> = glob_patterns
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
        .map(|file_path| File {
            path: file_path.clone(),
            hash: None,
            size: Some(fs::metadata(file_path).unwrap().len()),
        })
        .filter(|file| filters::is_file_gt_minsize(app_opts, file))
        .collect();

    Ok(files)
}

fn process_file_hash_index(file: &File) -> Result<File> {
    Ok(File {
        path: file.path.clone(),
        size: file.size,
        hash: Some(hash_file(&file.path).unwrap_or_default()),
    })
}

fn process_file_index(
    file: File,
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
            let processed_file = process_file_hash_index(&file).unwrap();
            let indexhash = processed_file.clone().hash.unwrap_or_default();

            store
                .entry(indexhash)
                .and_modify(|fileset| fileset.push(processed_file.clone()))
                .or_insert_with(|| vec![processed_file]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_by_size() {
        let files_to_index: Vec<File> = vec![
            File {
                path: "tf1.jpg".to_string(),
                size: Some(100_123),
                hash: None,
            },
            File {
                path: "tf2.png".to_string(),
                size: Some(100_123),
                hash: None,
            },
            File {
                path: "tf3.mp4".to_string(),
                size: Some(100_000_000),
                hash: None,
            },
        ];

        let duplicates_by_size = index_files(files_to_index, IndexCritera::Size).unwrap();
        let duplicate_paths = duplicates_by_size.view("100123", |_, value| {
            value
                .iter()
                .map(|f| f.clone().path)
                .collect::<Vec<String>>()
        }).unwrap();
        assert_eq!(duplicates_by_size.len(), 2);
        assert!(duplicates_by_size.contains_key("100123"));
        assert!(duplicate_paths.contains(&"tf1.jpg".to_string()));
        assert!(duplicate_paths.contains(&"tf2.png".to_string()));
    }
}
