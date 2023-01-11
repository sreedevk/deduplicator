use std::{collections::HashMap, fs, io};
use std::io::Write;

use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use humansize::{format_size, DECIMAL};
use itertools::Itertools;

use crate::app::file_manager;
use crate::database::File;
use crate::params::Params;
use prettytable::{format, row, Cell, Row, Table};

fn format_path(path: &str, opts: &Params) -> Result<String> {
    let display_path = path.replace(&opts.get_directory()?, "");
    let text_vec = display_path.chars().collect::<Vec<_>>();

    let display_range = if text_vec.len() > 32 {
        text_vec[(display_path.len() - 32)..]
            .iter()
            .collect::<String>()
    } else {
        display_path
    };

    Ok(format!("...{:<32}", display_range))
}

fn file_size(path: &String) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let formatted_size = format!("{:>12}", format_size(mdata.len(), DECIMAL));
    Ok(formatted_size)
}

fn modified_time(path: &String) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let modified_time: DateTime<Utc> = mdata.modified()?.into();

    Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn group_duplicates(duplicates: Vec<File>) -> HashMap<String, Vec<File>> {
    let mut duplicate_mapper: HashMap<String, Vec<File>> = HashMap::new();
    duplicates.into_iter().for_each(|file| {
        duplicate_mapper
            .entry(file.hash.clone())
            .and_modify(|value| value.push(file.clone()))
            .or_insert_with(|| vec![file]);
    });

    duplicate_mapper
}

fn print_meta_info(duplicates: &Vec<File>, opts: &Params) {
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
    print!("\nconfirm? [Y/n]: ");
    std::io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    match user_input.trim() {
        "Y" | "y" => Ok(true),
        _ => Ok(false)
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
        println!("Err: File Index Out of Bounds!");
        return process_group_action(duplicates, dup_index, dup_size, table);
    }

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    let files_to_delete = parsed_file_indices
        .into_iter()
        .map(|index| duplicates[index].clone());

    println!("\n{}", "The following files will be deleted:".red());
    files_to_delete.clone().enumerate().for_each(|(index, file)| {
        println!("{}: {}", index.to_string().blue(), file.path);
    });
    
    match scan_group_confirmation().unwrap() {
        true => { file_manager::delete_files(files_to_delete.collect_vec()); },
        false => println!("{}", "\nCancelled Delete Operation.".red())
    }
}

pub fn interactive(duplicates: Vec<File>, opts: &Params) {
    print_meta_info(&duplicates, opts);
    let grouped_duplicates = group_duplicates(duplicates);

    grouped_duplicates.iter().enumerate().for_each(|(gindex, (hash, group))| {
        let mut itable = Table::new();
        itable.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        itable.set_titles(row!["index", "filename", "size", "updated_at"]);
        group.iter().enumerate().for_each(|(index, file)| {
            itable.add_row(row![
                index,
                format_path(&file.path, opts).unwrap_or_default().blue(),
                file_size(&file.path).unwrap_or_default().red(),
                modified_time(&file.path).unwrap_or_default().yellow()
            ]);
        });

        process_group_action(group, gindex, grouped_duplicates.len(), itable);
    });
}

pub fn print(duplicates: Vec<File>, opts: &Params) {
    print_meta_info(&duplicates, opts);

    let mut output_table = Table::new();
    let grouped_duplicates: HashMap<String, Vec<File>> = group_duplicates(duplicates);

    output_table.set_titles(row!["hash", "duplicates"]);
    grouped_duplicates.iter().for_each(|(hash, group)| {
        let mut inner_table = Table::new();
        inner_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        group.iter().for_each(|file| {
            inner_table.add_row(row![
                format_path(&file.path, opts).unwrap_or_default().blue(),
                file_size(&file.path).unwrap_or_default().red(),
                modified_time(&file.path).unwrap_or_default().yellow()
            ]);
        });
        output_table.add_row(row![hash.green(), inner_table]);
    });

    output_table.printstd();
}
