use crate::database::File;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use humansize::{format_size, DECIMAL};
use std::{collections::HashMap, fs};
use crate::params::Params;

fn format_path(path: &String, opts: &Params) -> String {
    format!("...{}", path.replace(&opts.get_directory().unwrap(), ""))
}

fn file_size(path: &String) -> String {
    let mdata = fs::metadata(path).unwrap();
    let formatted_size = format_size(mdata.len(), DECIMAL);
    format!("{}", formatted_size)
}

fn modified_time(path: &String) -> String {
    let mdata = fs::metadata(path).unwrap();
    let modified_time: DateTime<Utc> = mdata.modified().unwrap().into();

    modified_time.format("%Y-%m-%d %H:%M:%S").to_string()
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
            .or_insert(vec![file]);
    });

    dup_index.into_iter().for_each(|(_, group)| {
        group.into_iter().for_each(|file| {
            println!(
                "| {0: <16} | {1: <35} | {2: <16} | {3: <32} |",
                file.hash.red(),
                format_path(&file.path, opts).yellow(),
                file_size(&file.path).blue(),
                modified_time(&file.path).blue()
            );
        });
        print_divider();
    });
}
