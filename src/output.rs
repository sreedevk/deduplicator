use crate::file_manager::{self, File};
use crate::params::Params;
use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use dashmap::DashMap;
use itertools::Itertools;
use prettytable::{format, row, Table};
use std::io::Write;
use std::{fs, io};
use unicode_segmentation::UnicodeSegmentation;

fn format_path(path: &str, opts: &Params) -> Result<String> {
    let display_path = path.replace(&opts.get_directory()?, "");
    let display_range = if display_path.chars().count() > 32 {
        display_path
            .graphemes(true)
            .collect::<Vec<&str>>()
            .into_iter()
            .rev()
            .take(32)
            .rev()
            .collect()
    } else {
        display_path
    };

    Ok(format!("...{:<32}", display_range))
}

fn file_size(file: &File) -> Result<String> {
    Ok(format!("{:>12}", bytesize::ByteSize::b(file.size.unwrap())))
}

fn modified_time(path: &String) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let modified_time: DateTime<Utc> = mdata.modified()?.into();

    Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn print_meta_info() {
    println!("Deduplicator v{}", std::env!("CARGO_PKG_VERSION"));
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

fn process_group_action(duplicates: &Vec<File>, dup_index: usize, dup_size: usize, table: Table) {
    println!("\nDuplicate Set {} of {}\n", dup_index + 1, dup_size);
    table.printstd();
    let files_to_delete = scan_group_instruction().unwrap_or_default();
    let parsed_file_indices = files_to_delete
        .trim()
        .split(',')
        .filter(|element| !element.is_empty())
        .map(|index| index.parse::<usize>().unwrap_or_default())
        .collect_vec();

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
            println!("{}: {}", index.to_string().blue(), file.path);
        });

    match scan_group_confirmation().unwrap() {
        true => {
            file_manager::delete_files(files_to_delete.collect_vec()).ok();
        }
        false => println!("{}", "\nCancelled Delete Operation.".red()),
    }
}

pub fn interactive(duplicates: DashMap<String, Vec<File>>, opts: &Params) {
    print_meta_info();

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
        .sorted_unstable_by_key(|f| {
            -(f.1.first().and_then(|ff| ff.size).unwrap_or_default() as i64)
        }) // sort by descending file size in interactive mode
        .enumerate()
        .for_each(|(gindex, (_, group))| {
            let mut itable = Table::new();
            itable.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            itable.set_titles(row!["index", "filename", "size", "updated_at"]);
            group.iter().enumerate().for_each(|(index, file)| {
                itable.add_row(row![
                    index,
                    format_path(&file.path, opts).unwrap_or_default().blue(),
                    file_size(&file).unwrap_or_default().red(),
                    modified_time(&file.path).unwrap_or_default().yellow()
                ]);
            });

            process_group_action(&group, gindex, duplicates.len(), itable);
        });
}

pub fn print(duplicates: DashMap<String, Vec<File>>, opts: &Params) {
    print_meta_info();

    if duplicates.is_empty() {
        println!(
            "\n{}",
            "No duplicates found matching your search criteria.".green()
        );
        return;
    }

    let mut output_table = Table::new();
    output_table.set_titles(row!["hash", "duplicates"]);
    duplicates
        .into_iter()
        .sorted_unstable_by_key(|f| f.1.first().and_then(|ff| ff.size).unwrap_or_default()) // sort by ascending size
        .for_each(|(hash, group)| {
            let mut inner_table = Table::new();
            inner_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            group.iter().for_each(|file| {
                inner_table.add_row(row![
                    format_path(&file.path, opts).unwrap_or_default().blue(),
                    file_size(&file).unwrap_or_default().red(),
                    modified_time(&file.path).unwrap_or_default().yellow()
                ]);
            });
            output_table.add_row(row![hash.green(), inner_table]);
        });

    output_table.printstd();
}
