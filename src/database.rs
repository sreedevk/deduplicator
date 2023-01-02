use anyhow::Result;
use crate::params::Params;

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub hash: String,
}

pub fn get_connection(args: &Params) -> Result<sqlite::Connection, sqlite::Error> {
    let connection_url = match args.nocache {
        false => "/tmp/deduplicator.db",
        true => ":memory:"
    };

    sqlite::open(connection_url).and_then(|conn| {
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
    let result = connection.execute(query)?;

    Ok(result)
}

pub fn indexed_paths(connection: &sqlite::Connection) -> Result<Vec<File>> {
    let query = format!(
        "SELECT * FROM files"
        );

    let result: Vec<File> = connection
        .prepare(query)?
        .into_iter()
        .map(|row_result| row_result.unwrap())
        .map(|row| {
            let path = row.read::<&str, _>("file_identifier").to_string();
            let hash = row.read::<i64, _>("hash").to_string();
            File { path, hash }
        })
    .collect();

    Ok(result)
}

pub fn duplicate_hashes(connection: &sqlite::Connection, path: &String) -> Result<Vec<File>> {
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
        ", path
    );

    let result: Vec<File> = connection
        .prepare(query)?
        .into_iter()
        .map(|row_result| row_result.unwrap())
        .map(|row| {
            let path = row.read::<&str, _>("file_identifier").to_string();
            let hash = row.read::<i64, _>("hash").to_string();
            File { path, hash }
        })
        .collect();

    Ok(result)
}
