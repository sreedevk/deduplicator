use crate::{file_manager::File, filters, params::Params};
use anyhow::Result;
use dashmap::DashMap;
use fxhash::hash64 as hasher;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::hash::Hasher;
use std::time::Duration;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
enum IndexCritera {
    Size,
    Hash,
}

// pub fn duplicates(app_opts: &Params) -> Result<DashMap<String, Vec<File>>> {
//     let scan_results = scan(app_opts)?;
//     let size_index_store = index_files(scan_results, IndexCritera::Size)?;
//
//     let sizewize_duplicate_files = size_index_store
//         .into_par_iter()
//         .filter(|(_, files)| files.len() > 1)
//         .map(|(_, files)| files)
//         .flatten()
//         .collect::<Vec<File>>();
//
//     if sizewize_duplicate_files.len() > 1 {
//         let hash_index_store = index_files(sizewize_duplicate_files, IndexCritera::Hash)?;
//         let duplicate_files = hash_index_store
//             .into_par_iter()
//             .filter(|(_, files)| files.len() > 1)
//             .collect();
//
//         Ok(duplicate_files)
//     } else {
//         Ok(DashMap::new())
//     }
// }

// fn scan(app_opts: &Params) -> Result<Vec<File>> {
//     let walker = app_opts.get_glob_walker()?;
//     let progress = ProgressBar::new_spinner();
//     let progress_style =
//         ProgressStyle::with_template("{spinner:.green} [mapping paths] {pos} paths")?;
//     progress.set_style(progress_style);
//     progress.enable_steady_tick(Duration::from_millis(50));
//
//     let files = walker
//         .progress_with(progress)
//         .filter_map(Result::ok)
//         .map(|file| file.into_path())
//         .filter(|fpath| fpath.is_file())
//         .collect::<Vec<PathBuf>>();
//
//     let scan_progress = ProgressBar::new(files.len() as u64);
//     let scan_progress_style = ProgressStyle::with_template(
//         "{spinner:.green} [processing mapped paths] [{wide_bar:.cyan/blue}] {pos}/{len} files",
//     )?;
//     scan_progress.set_style(scan_progress_style);
//     scan_progress.enable_steady_tick(Duration::from_millis(50));
//
//     let scan_results = files
//         .into_par_iter()
//         .progress_with(scan_progress)
//         .map(|fpath| File {
//             path: fpath.clone(),
//             hash: None,
//             size: Some(
//                 fs::metadata(fpath)
//                     .map(|metadata| metadata.len())
//                     .unwrap_or_default(),
//             ),
//         })
//         .filter(|file| filters::is_file_gt_min_size(app_opts, file))
//         .collect();
//
//     Ok(scan_results)
// }

// fn process_file_index(
//     mut file: File,
//     store: &DashMap<String, Vec<File>>,
//     index_criteria: IndexCritera,
// ) {
//     match index_criteria {
//         IndexCritera::Size => {
//             store
//                 .entry(file.size.unwrap_or_default().to_string())
//                 .and_modify(|fileset| fileset.push(file.clone()))
//                 .or_insert_with(|| vec![file]);
//         }
//         IndexCritera::Hash => {
//             file.hash = Some(hash_file(&file.path).unwrap_or_default());
//             store
//                 .entry(file.clone().hash.unwrap_or_default())
//                 .and_modify(|fileset| fileset.push(file.clone()))
//                 .or_insert_with(|| vec![file]);
//         }
//     }
// }
//
// fn index_files(
//     files: Vec<File>,
//     index_criteria: IndexCritera,
// ) -> Result<DashMap<String, Vec<File>>> {
//     let store: DashMap<String, Vec<File>> = DashMap::new();
//     let index_progress = ProgressBar::new(files.len() as u64);
//     let index_progress_style = ProgressStyle::with_template(
//         "{spinner:.green} [indexing files] [{wide_bar:.cyan/blue}] {pos}/{len} files",
//     )?;
//     index_progress.set_style(index_progress_style);
//     index_progress.enable_steady_tick(Duration::from_millis(50));
//
//     files
//         .into_par_iter()
//         .progress_with(index_progress)
//         .for_each(|file| process_file_index(file, &store, index_criteria));
//
//     Ok(store)
// }
//
// fn incremental_hashing(filepath: &Path) -> Result<String> {
//     let file = fs::File::open(filepath)?;
//     let fmap = unsafe { Mmap::map(&file)? };
//     let mut inchasher = fxhash::FxHasher::default();
//
//     fmap.chunks(1_000_000)
//         .for_each(|mega| inchasher.write(mega));
//
//     Ok(format!("{}", inchasher.finish()))
// }
//
// fn standard_hashing(filepath: &Path) -> Result<String> {
//     let file = fs::read(filepath)?;
//     Ok(hasher(&*file).to_string())
// }
//
// fn hash_file(filepath: &Path) -> Result<String> {
//     let filemeta = fs::metadata(filepath)?;
//
//     // NOTE: USE INCREMENTAL HASHING ONLY FOR FILES > 100MB
//     match filemeta.len() < 100_000_000 {
//         true => standard_hashing(filepath),
//         false => incremental_hashing(filepath),
//     }
// }
