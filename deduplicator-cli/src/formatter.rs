pub struct Formatter;
use crate::params::Params;
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use dashmap::DashMap;
use deduplicator_core::fileinfo::FileInfo;
use indicatif::{ProgressBar, ProgressStyle};
use pathdiff::diff_paths;
use prettytable::{format, row, Table};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

impl Formatter {
    fn human_path(file: &FileInfo, app_args: &Params, min_path_length: usize) -> Result<String> {
        let base_directory: PathBuf = app_args.get_directory()?.to_path_buf();
        let relative_path = diff_paths(file.path.clone(), base_directory).unwrap_or_default();

        let formatted_path = format!(
            "{:<0width$}",
            relative_path.to_str().unwrap_or_default().to_string(),
            width = min_path_length
        );

        Ok(formatted_path)
    }

    fn human_filesize(file: &FileInfo) -> Result<String> {
        Ok(format!("{:>12}", bytesize::ByteSize::b(file.size)))
    }

    fn human_mtime(file: &FileInfo) -> Result<String> {
        let modified_time: DateTime<Utc> = file.filemeta.modified()?.into();
        Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    pub fn generate_table(raw: Vec<FileInfo>, app_args: &Params) -> Result<Table> {
        let basepath_length = app_args.get_directory()?.to_str().unwrap_or_default().len();
        let max_filepath_length = raw
            .iter()
            .map(|file| file.path.to_str().unwrap_or_default().len())
            .max()
            .unwrap_or_default();
        let min_path_length = max_filepath_length - basepath_length;

        let duplicates_table: DashMap<String, Vec<FileInfo>> = DashMap::new();
        raw.clone()
            .into_par_iter()
            .map(|file| file.hash())
            .filter_map(Result::ok)
            .for_each(|file| {
                duplicates_table
                    .entry(file.hash.clone().unwrap_or_default())
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        let mut output_table = Table::new();
        let progress_bar = ProgressBar::new(raw.len() as u64);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        let progress_style = ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [generating output] [{wide_bar:.cyan/blue}] {pos}/{len} files",
            )
            .unwrap();

        progress_bar.set_style(progress_style);
        output_table.set_titles(row!["hash", "duplicates"]);

        duplicates_table.into_iter().for_each(|(hash, group)| {
            let mut inner_table = Table::new();
            inner_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            group.iter().for_each(|file| {
                inner_table.add_row(row![
                    Self::human_path(&file, &app_args, min_path_length)
                        .unwrap_or_default()
                        .blue(),
                    Self::human_filesize(&file).unwrap_or_default().red(),
                    Self::human_mtime(&file).unwrap_or_default().yellow()
                ]);
            });
            output_table.add_row(row![hash.green(), inner_table]);
        });

        Ok(output_table)
    }

    pub fn print(raw: Vec<FileInfo>, app_args: &Params) -> Result<()> {
        let output_table = Self::generate_table(raw, app_args)?;
        output_table.printstd();

        Ok(())
    }
}
