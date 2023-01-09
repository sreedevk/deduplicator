use std::env::temp_dir;

use anyhow::Result;

use crate::params::Params;

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub hash: String,
}

fn db_connection_url(args: &Params) -> String {
    match args.nocache {
        true => String::from(":memory:"),
        false => {
            let temp_dir_path = temp_dir();
            format!("{}/deduplicator.db", temp_dir_path.display())
        }
    }
}

pub fn get_connection(args: &Params) -> Result<sqlite::Connection, sqlite::Error> {
    sqlite::open(db_connection_url(args)).and_then(|conn| {
        setup(&conn).ok();
        Ok(conn)
    })
}

pub fn setup(connection: &sqlite::Connection) -> Result<()> {
    let query = "CREATE TABLE files (file_identifier STRING, hash STRING)";
    connection.execute(query).ok();
    Ok(())
}

pub fn put(file: &File, connection: &sqlite::Connection) -> Result<()> {
    let query = format!(
        "INSERT INTO files (file_identifier, hash) VALUES (\"{}\", \"{}\")",
        file.path, file.hash
    );
    connection.execute(query)?;
    Ok(())
}

pub fn indexed_paths(connection: &sqlite::Connection) -> Result<Vec<File>> {
    let query = "SELECT * FROM files";

    let result: Vec<File> = connection
        .prepare(query)?
        .into_iter()
        .filter_map(|row_result| row_result.ok())
        .map(|row| {
            let path = row.read::<&str, _>("file_identifier").to_string();
            let hash = row.read::<i64, _>("hash").to_string();
            File { path, hash }
        })
        .collect();

    Ok(result)
}

pub fn duplicate_hashes(connection: &sqlite::Connection, path: &str) -> Result<Vec<File>> {
    let query = format!(
        "
            SELECT a.* FROM files a
            JOIN (SELECT file_identifier, hash, COUNT(*)
            FROM files
            GROUP BY hash
            HAVING count(*) > 1 ) b
            ON a.hash = b.hash
            WHERE a.file_identifier LIKE \"{}%\"
            ORDER BY a.file_identifier
        ",
        path
    );

    let result: Vec<File> = connection
        .prepare(query)?
        .into_iter()
        .filter_map(|row_result| row_result.ok())
        .map(|row| {
            let path = row.read::<&str, _>("file_identifier").to_string();
            let hash = row.read::<i64, _>("hash").to_string();
            File { path, hash }
        })
        .collect();

    Ok(result)
}
