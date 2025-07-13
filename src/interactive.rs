use crate::{fileinfo::FileInfo, formatter::Formatter, params::Params};
use anyhow::Result;
use dashmap::DashMap;
use prettytable::{format, row, Table};
use std::{
    io::{self, Write},
    sync::Arc,
};

pub struct Interactive;

impl Interactive {
    pub fn init(result: Arc<DashMap<String, Vec<FileInfo>>>, app_args: &Params) -> Result<()> {
        result.clone().iter().enumerate().for_each(|(gindex, i)| {
            let group = i.value();
            let mut itable = Table::new();
            itable.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            itable.set_titles(row!["index", "filename", "size", "updated_at"]);
            let max_path_size = group
                .iter()
                .map(|f| f.path.clone().into_os_string().len())
                .max()
                .unwrap_or_default();

            group.iter().enumerate().for_each(|(index, file)| {
                itable.add_row(row![
                    index,
                    Formatter::human_path(file, app_args, max_path_size).unwrap_or_default(),
                    Formatter::human_filesize(file).unwrap_or_default(),
                    Formatter::human_mtime(file).unwrap_or_default()
                ]);
            });

            Self::process_group_action(group, gindex, result.len(), itable);
        });

        Ok(())
    }

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

    pub fn process_group_action(
        duplicates: &Vec<FileInfo>,
        dup_index: usize,
        dup_size: usize,
        table: Table,
    ) {
        println!("\nDuplicate Set {} of {}\n", dup_index + 1, dup_size);
        table.printstd();
        let files_to_delete = Self::scan_group_instruction().unwrap_or_default();
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
            println!("Err: File Index Out of Bounds!");
            return Self::process_group_action(duplicates, dup_index, dup_size, table);
        }

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        if parsed_file_indices.is_empty() {
            return;
        }

        let files_to_delete = parsed_file_indices
            .into_iter()
            .map(|index| duplicates[index].clone());

        println!("\nThe following files will be deleted:");
        files_to_delete
            .clone()
            .enumerate()
            .for_each(|(index, file)| {
                println!("{}: {}", index, file.path.display());
            });

        match Self::scan_group_confirmation().unwrap() {
            true => {
                files_to_delete.into_iter().for_each(|file| {
                    match std::fs::remove_file(file.path.clone()) {
                        Ok(_) => println!("DELETED: {}", file.path.display()),
                        Err(_) => println!("FAILED: {}", file.path.display()),
                    }
                });
            }
            false => println!("\nCancelled Delete Operation."),
        }
    }
}
