use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use pathdiff::diff_paths;
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc};

const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

pub struct Formatter;
impl Formatter {
    pub fn human_path(file: &FileInfo, aargs: &Params, min_path_length: usize) -> Result<String> {
        let base_directory: PathBuf = aargs.get_directory()?;
        let relative_path = diff_paths(&file.path, base_directory).unwrap_or_default();

        let formatted_path = format!(
            "{:<0width$}",
            relative_path.to_str().unwrap_or_default().to_string(),
            width = min_path_length
        );

        Ok(formatted_path)
    }

    pub fn human_filesize(file: &FileInfo) -> Result<String> {
        Ok(format!("{:>12}", bytesize::ByteSize::b(file.size)))
    }

    pub fn human_mtime(file: &FileInfo) -> Result<String> {
        let modified_time: DateTime<Utc> = file.modified.into();
        Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    pub fn print(raw: Arc<DashMap<u128, Vec<FileInfo>>>, max_path_len: u64, aargs: &Params) {
        print!("{}", "\n".repeat(if aargs.progress { 2 } else { 1 })); // spacing

        if raw.is_empty() {
            println!("No duplicates found matching your search criteria.");
        } else {
            raw.par_iter().for_each(|sref| {
                let mut ostring = format!("{}{:32x}{}\n", YELLOW, sref.key(), RESET);
                let subfields = sref
                    .value()
                    .par_iter()
                    .map(|finfo| {
                        format!(
                            "├─ {}\t{}\t{}\n",
                            Self::human_path(finfo, aargs, max_path_len as usize)
                                .expect("path formatting failed."),
                            Self::human_filesize(finfo).expect("filesize formatting failed."),
                            Self::human_mtime(finfo).expect("modified time formatting failed.")
                        )
                    })
                    .collect::<String>();

                ostring.push_str(&subfields);

                println!("{ostring}");
            });
        }
    }
}
