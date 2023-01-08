use std::{collections::HashMap, fs};

use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use humansize::{format_size, DECIMAL};

use crate::database::File;
use crate::params::Params;

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

    Ok(format!("...{}", display_range))
}

fn file_size(path: &String) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let formatted_size = format_size(mdata.len(), DECIMAL);
    Ok(formatted_size)
}

fn modified_time(path: &String) -> Result<String> {
    let mdata = fs::metadata(path)?;
    let modified_time: DateTime<Utc> = mdata.modified()?.into();

    Ok(modified_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn print_divider() {
    println!("-------------------+-------------------------------------+------------------+----------------------------------+");
}

pub fn print(duplicates: Vec<File>, opts: &Params) {
    print_divider();
    println!(
        "| {0: <16} | {1: <35} | {2: <16} | {3: <32} |",
        "hash", "filename", "size", "updated_at"
    );
    print_divider();

    let mut dup_index: HashMap<String, Vec<File>> = HashMap::new();

    duplicates.into_iter().for_each(|file| {
        dup_index
            .entry(file.hash.clone())
            .and_modify(|value| value.push(file.clone()))
            .or_insert_with(|| vec![file]);
    });

    dup_index.into_iter().for_each(|(_, group)| {
        group.into_iter().for_each(|file| {
            let Ok(path) = format_path(&file.path, opts) else {
                return;
            };
            let Ok(file_size) = file_size(&file.path) else {
                return;
            };
            let Ok(modified_time) = modified_time(&file.path) else {
                return;
            };
            println!(
                "| {0: <16} | {1: <35} | {2: <16} | {3: <32} |",
                file.hash.red(),
                path.yellow(),
                file_size.blue(),
                modified_time.blue()
            );
        });
        print_divider();
    });
}
