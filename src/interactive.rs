use crate::formatter::Formatter;
use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use colored::Colorize;
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle};
use prettytable::{format, row, Table};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    io::{self, Write},
    time::Duration,
};

pub fn scan_group_confirmation() -> Result<bool> {
    print!("\nconfirm? [y/N]: ");
    std::io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    match user_input.trim() {
        "Y" | "y" => Ok(true),
        _ => Ok(false),
    }
}

pub fn scan_group_instruction() -> Result<String> {
    println!("\nEnter the indices of the files you want to delete.");
    println!("You can enter multiple files using commas to seperate file indices.");
    println!("example: 1,2");
    print!("\n> ");
    std::io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    Ok(user_input)
}

pub fn init(result: Vec<FileInfo>, app_args: &Params) -> Result<()> {
    let basepath_length = app_args.get_directory()?.to_str().unwrap_or_default().len();
    let max_filepath_length = result
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
    let progress_bar = ProgressBar::new(result.len() as u64);
    progress_bar.set_style(progress_style);
    progress_bar.enable_steady_tick(Duration::from_millis(50));
    progress_bar.set_message("reconciling data");

    let duplicates: DashMap<String, Vec<FileInfo>> = DashMap::new();
    result
        .into_par_iter()
        .progress_with(progress_bar)
        .with_finish(ProgressFinish::WithMessage(Cow::from("data reconciled")))
        .map(|file| file.hash())
        .filter_map(Result::ok)
        .for_each(|file| {
            duplicates
                .entry(file.hash.clone().unwrap_or_default())
                .and_modify(|fileset| fileset.push(file.clone()))
                .or_insert_with(|| vec![file]);
        });

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
                    Formatter::human_path(file, app_args, min_path_length)
                        .unwrap_or_default()
                        .blue(),
                    Formatter::human_filesize(file).unwrap_or_default().red(),
                    Formatter::human_mtime(file).unwrap_or_default().yellow()
                ]);
            });

            process_group_action(&group, gindex, duplicates.len(), itable);
        });

    Ok(())
}

pub fn process_group_action(
    duplicates: &Vec<FileInfo>,
    dup_index: usize,
    dup_size: usize,
    table: Table,
) {
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
            files_to_delete.into_iter().for_each(|file| {
                match std::fs::remove_file(file.path.clone()) {
                    Ok(_) => println!("{}: {}", "DELETED".green(), file.path.display()),
                    Err(_) => println!("{}: {}", "FAILED".red(), file.path.display()),
                }
            });
        }
        false => println!("{}", "\nCancelled Delete Operation.".red()),
    }
}
