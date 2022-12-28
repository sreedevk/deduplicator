use crate::database::File;
use itertools::Itertools;
use colored::Colorize;
use std::fs;
use chrono::offset::Utc;
use chrono::DateTime;
use humansize::{format_size, DECIMAL};

fn format_path(path: &String) -> String {
    let stringlen = path.len();
    format!("...{}", String::from(&path[(stringlen - 32)..]))
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

pub fn print(duplicates: Vec<File>) {
    print_divider();
    println!(
        "| {0: <16} | {1: <35} | {2: <16} | {3: <32} |",
        "hash", "filename", "size", "updated_at"
    );
    print_divider();

    duplicates
        .into_iter()
        .group_by(|record| record.hash.clone())
        .into_iter()
        .for_each(|(_, group)| {
            group
                .into_iter()
                .for_each(|file| {
                    println!(
                        "| {0: <16} | {1: <35} | {2: <16} | {3: <32} |",
                        &file.hash[0..16].red(),
                        format_path(&file.path).yellow(),
                        file_size(&file.path).blue(),
                        modified_time(&file.path).blue()
                    );
                });
            print_divider();
        });
}
