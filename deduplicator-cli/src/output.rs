use crate::file_manager;
use crate::params::Params;
use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use dashmap::DashMap;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use prettytable::{format, row, Table};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::{fs, io};
use deduplicator_core::fileinfo::FileInfo;

fn format_path(path: &Path) -> Result<String> {
    let display_path = path.to_string_lossy();
    Ok(format!("{display_path}"))
}

fn file_size(file: &FileInfo) -> Result<String> {
    Ok(format!("{:>12}", bytesize::ByteSize::b(file.size)))
}

fn modified_time(path: &Path) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let modified_time: DateTime<Utc> = mdata.modified()?.into();

    Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn scan_group_instruction() -> Result<String> {
    println!("\nEnter the indices of the files you want to delete.");
    println!("You can enter multiple files using commas to seperate file indices.");
    println!("example: 1,2");
    print!("\n> ");
    std::io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    Ok(user_input)
}

fn scan_group_confirmation() -> Result<bool> {
    print!("\nconfirm? [y/N]: ");
    std::io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    match user_input.trim() {
        "Y" | "y" => Ok(true),
        _ => Ok(false),
    }
}

fn process_group_action(duplicates: &Vec<FileInfo>, dup_index: usize, dup_size: usize, table: Table) {
    println!("\nDuplicate Set {} of {}\n", dup_index + 1, dup_size);
    table.printstd();
    let files_to_delete = scan_group_instruction().unwrap_or_default();
    let parsed_file_indices = files_to_delete
        .trim()
        .split(',')
        .filter(|element| !element.is_empty())
        .map(|index| index.parse::<usize>().unwrap_or_default())
        .collect::<Vec<usize>>();

    if parsed_file_indices
        .clone()
        .into_iter()
        .any(|index| index > (duplicates.len() - 1))
    {
        println!("{}", "Err: File Index Out of Bounds!".red());
        return process_group_action(duplicates, dup_index, dup_size, table);
    }

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    if parsed_file_indices.is_empty() {
        return;
    }

    let files_to_delete = parsed_file_indices
        .into_iter()
        .map(|index| duplicates[index].clone());

    println!("\n{}", "The following files will be deleted:".red());
    files_to_delete
        .clone()
        .enumerate()
        .for_each(|(index, file)| {
            println!("{}: {}", index.to_string().blue(), file.path.display());
        });

    match scan_group_confirmation().unwrap() {
        true => {
            file_manager::delete_files(files_to_delete.collect::<Vec<FileInfo>>()).ok();
        }
        false => println!("{}", "\nCancelled Delete Operation.".red()),
    }
}

pub fn interactive(duplicates: DashMap<String, Vec<FileInfo>>, opts: &Params) {
    if duplicates.is_empty() {
        println!(
            "\n{}",
            "No duplicates found matching your search criteria.".green()
        );
        return;
    }

    duplicates
        .clone()
        .into_iter()
        .enumerate()
        .for_each(|(gindex, (_, group))| {
            let mut itable = Table::new();
            itable.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            itable.set_titles(row!["index", "filename", "size", "updated_at"]);
            group.iter().enumerate().for_each(|(index, file)| {
                itable.add_row(row![
                    index,
                    format_path(&file.path).unwrap_or_default().blue(),
                    file_size(file).unwrap_or_default().red(),
                    modified_time(&file.path).unwrap_or_default().yellow()
                ]);
            });

            process_group_action(&group, gindex, duplicates.len(), itable);
        });
}

pub fn print(duplicates: DashMap<String, Vec<FileInfo>>) {
    if duplicates.is_empty() {
        println!(
            "\n{}",
            "No duplicates found matching your search criteria.".green()
        );
        return;
    }

    let mut output_table = Table::new();
    let progress_bar = ProgressBar::new(duplicates.len() as u64);
    progress_bar.enable_steady_tick(Duration::from_millis(50));
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [generating output] [{wide_bar:.cyan/blue}] {pos}/{len} files")
        .unwrap();

    progress_bar.set_style(progress_style);
    output_table.set_titles(row!["hash", "duplicates"]);

}
