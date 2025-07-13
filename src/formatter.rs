use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle};
use pathdiff::diff_paths;
use prettytable::{format, row, Row, Table};
use rayon::prelude::*;
use std::{borrow::Cow, path::PathBuf, sync::Arc, time::Duration};

pub struct Formatter;
impl Formatter {
    pub fn human_path(
        file: &FileInfo,
        app_args: &Params,
        min_path_length: usize,
    ) -> Result<String> {
        let base_directory: PathBuf = app_args.get_directory()?;
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

    pub fn gen_sub_tbl(items: Vec<FileInfo>, app_args: &Params, max_path_len: u64) -> Table {
        let mut inner_table = Table::new();
        inner_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        items.iter().for_each(|file| {
            inner_table.add_row(row![
                Self::human_path(file, app_args, max_path_len as usize).unwrap_or_default(),
                Self::human_filesize(file).unwrap_or_default(),
                Self::human_mtime(file).unwrap_or_default()
            ]);
        });
        inner_table
    }

    pub fn generate_table(
        raw: Arc<DashMap<u128, Vec<FileInfo>>>,
        mpath_len: u64,
        args: &Params,
    ) -> Result<Table> {
        let progress_bar = match args.progress {
            true => ProgressBar::new_spinner(),
            false => ProgressBar::hidden(),
        };

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("generating output");

        let rows = raw
            .par_iter_mut()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("output generated")))
            .filter(|i| i.value().len() > 1)
            .map(|i| {
                row![
                    i.key(),
                    Self::gen_sub_tbl(i.value().to_vec(), args, mpath_len)
                ]
            })
            .collect::<Vec<Row>>();

        let mut output_table = Table::new();
        output_table.set_titles(row!["hash", "duplicates"]);
        output_table.extend(rows);
        Ok(output_table)
    }

    pub fn print(
        raw: Arc<DashMap<u128, Vec<FileInfo>>>,
        max_path_len: u64,
        app_args: &Params,
    ) -> Result<()> {
        if raw.is_empty() {
            println!("\n\nNo duplicates found matching your search criteria.\n");
            return Ok(());
        }

        let output_table = Self::generate_table(raw, max_path_len, app_args)?;
        output_table.printstd();

        Ok(())
    }
}
