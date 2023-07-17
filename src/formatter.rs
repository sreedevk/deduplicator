pub struct Formatter;
use crate::fileinfo::FileInfo;
use crate::params::Params;
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use dashmap::DashMap;
use indicatif::{
    ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle,
};
use pathdiff::diff_paths;
use prettytable::{format, row, Table};
use rayon::prelude::*;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

impl Formatter {
    pub fn human_path(file: &FileInfo, app_args: &Params, min_path_length: usize) -> Result<String> {
        let base_directory: PathBuf = app_args.get_directory()?.to_path_buf();
        let relative_path = diff_paths(file.path.clone(), base_directory).unwrap_or_default();

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

        let min_path_length = if max_filepath_length > basepath_length {
            max_filepath_length - basepath_length
        } else {
            0
        };

        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let progress_bar = ProgressBar::new(raw.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("reconciling data");

        let duplicates_table: DashMap<String, Vec<FileInfo>> = DashMap::new();
        raw.clone()
            .into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("data reconciled")))
            .map(|file| file.hash())
            .filter_map(Result::ok)
            .for_each(|file| {
                duplicates_table
                    .entry(file.hash.clone().unwrap_or_default())
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        let mut output_table = Table::new();
        output_table.set_titles(row!["hash", "duplicates"]);

        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;

        let progress_bar = ProgressBar::new(duplicates_table.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("generating output");

        duplicates_table
            .into_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("output generated")))
            .for_each(|(hash, group)| {
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
        if raw.is_empty() {
            println!(
                "\n\n{}\n",
                "No duplicates found matching your search criteria.".green()
            );
            return Ok(());
        }

        let output_table = Self::generate_table(raw, app_args)?;
        output_table.printstd();

        Ok(())
    }
}
