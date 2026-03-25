use crate::fileinfo::FileInfo;
use crate::formatter::Formatter;
use crate::params::{KeepStrategy, Params};
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;

const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";

pub struct Bulk;

impl Bulk {
    pub fn execute(
        result: Arc<DashMap<u128, Vec<FileInfo>>>,
        strategy: &KeepStrategy,
        delete: bool,
        app_args: &Params,
        max_path_len: usize,
    ) -> Result<()> {
        if result.is_empty() {
            println!("No duplicates found matching your search criteria.");
            return Ok(());
        }

        let mut found_duplicates = false;

        for sref in result.iter() {
            let group = sref.value();
            if group.len() <= 1 {
                continue;
            }

            found_duplicates = true;

            let mut files: Vec<&FileInfo> = group.iter().collect();
            files.sort_by_key(|f| f.modified);

            let keeper_mtime = match strategy {
                KeepStrategy::Newest => files.last().unwrap().modified,
                KeepStrategy::Oldest => files.first().unwrap().modified,
            };

            if delete {
                Self::execute_delete(&files, keeper_mtime)?;
            } else {
                Self::print_dry_run(&files, keeper_mtime, sref.key(), app_args, max_path_len)?;
            }
        }

        if !found_duplicates {
            println!("No duplicates found matching your search criteria.");
        }

        Ok(())
    }

    fn print_dry_run(
        files: &[&FileInfo],
        keeper_mtime: std::time::SystemTime,
        hash: &u128,
        app_args: &Params,
        max_path_len: usize,
    ) -> Result<()> {
        let mut output = format!("{}{:32x}{}\n", YELLOW, hash, RESET);

        let mut first_keep = true;
        for (i, file) in files.iter().enumerate() {
            let nodechar = if i == files.len() - 1 { "└─" } else { "├─" };
            let annotation = if file.modified == keeper_mtime {
                if first_keep {
                    first_keep = false;
                    format!("{}[KEEP]{}", GREEN, RESET)
                } else {
                    format!("{}[KEEP - same mtime]{}", GREEN, RESET)
                }
            } else {
                format!("{}[DELETE]{}", RED, RESET)
            };

            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                nodechar,
                Formatter::human_path(file, app_args, max_path_len)?,
                Formatter::human_filesize(file)?,
                Formatter::human_mtime(file)?,
                annotation,
            ));
        }

        println!("{output}");
        Ok(())
    }

    fn execute_delete(
        files: &[&FileInfo],
        keeper_mtime: std::time::SystemTime,
    ) -> Result<()> {
        let mut first_keep = true;
        for file in files {
            if file.modified == keeper_mtime {
                if first_keep {
                    first_keep = false;
                    println!("KEPT:    {}", file.path.display());
                } else {
                    println!("KEPT:    {} (same mtime, skipped)", file.path.display());
                }
            } else {
                match std::fs::remove_file(&file.path) {
                    Ok(_) => println!("DELETED: {}", file.path.display()),
                    Err(_) => println!("FAILED:  {}", file.path.display()),
                }
            }
        }
        Ok(())
    }
}
