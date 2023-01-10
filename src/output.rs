use std::{collections::HashMap, fs};

use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use colored::Colorize;
use humansize::{format_size, DECIMAL};

use crate::database::File;
use crate::params::Params;
use prettytable::{row, Cell, Row, Table};

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

pub fn print(duplicates: Vec<File>, opts: &Params) {
    let mut output_table = Table::new();
    let mut dup_index: HashMap<String, Vec<File>> = HashMap::new();
    output_table.add_row(row!["hash", "duplicates"]);
    duplicates.into_iter().for_each(|file| {
        dup_index
            .entry(file.hash.clone())
            .and_modify(|value| value.push(file.clone()))
            .or_insert_with(|| vec![file]);
    });

    dup_index.iter().for_each(|(hash, group)| {
        let mut inner_table = Table::new();
        inner_table.add_row(row!["filename", "size", "updated_at"]);
        group.iter().for_each(|file| {
            inner_table.add_row(row![
                format_path(&file.path, opts).unwrap_or_default().blue(),
                file_size(&file.path).unwrap_or_default().red(),
                modified_time(&file.path).unwrap_or_default().yellow()
            ]);
        });
        output_table.add_row(row![hash, inner_table]);
    });

    output_table.printstd();
}
