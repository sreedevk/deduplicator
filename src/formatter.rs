pub struct Formatter;
use crate::fileinfo::FileInfo;
use crate::params::Params;
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use pathdiff::diff_paths;
use prettytable::{format, row, Table};
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

impl Formatter {
    pub fn human_path(
        file: &FileInfo,
        app_args: &Params,
        min_path_length: usize,
    ) -> Result<String> {
        let base_directory: PathBuf = app_args.get_directory()?;
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

    pub fn generate_table(raw: DashMap<String, Vec<FileInfo>>, max_path_len: usize, app_args: &Params) -> Result<Table> {
        let mut output_table = Table::new();
        output_table.set_titles(row!["hash", "duplicates"]);

        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;

        let progress_bar = ProgressBar::new(raw.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("generating output");

        raw.into_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("output generated")))
            .for_each(|(hash, group)| {
                let mut inner_table = Table::new();
                inner_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                group.iter().for_each(|file| {
                    inner_table.add_row(row![
                        Self::human_path(file, app_args, max_path_len).unwrap_or_default(),
                        Self::human_filesize(file).unwrap_or_default(),
                        Self::human_mtime(file).unwrap_or_default()
                    ]);
                });

                output_table.add_row(row![hash, inner_table]);
            });

        Ok(output_table)
    }

    pub fn print(raw: DashMap<String, Vec<FileInfo>>, max_path_len: usize, app_args: &Params) -> Result<()> {
        if raw.is_empty() {
            println!("\n\nNo duplicates found matching your search criteria.\n");
            return Ok(());
        }

        let output_table = Self::generate_table(raw, max_path_len, app_args)?;
        output_table.printstd();

        Ok(())
    }
}
