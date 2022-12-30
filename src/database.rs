use anyhow::Result;

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub hash: String,
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
            let hash = row.read::<&str, _>("hash").to_string();
            File { path, hash }
        })
    .collect();

    Ok(result)
}

pub fn duplicate_hashes(connection: &sqlite::Connection) -> Result<Vec<File>> {
    let query = format!(
        " 
            SELECT a.* FROM files a
            JOIN (SELECT file_identifier, hash, COUNT(*)
            FROM files 
            GROUP BY hash
            HAVING count(*) > 1 ) b
            ON a.hash = b.hash
            ORDER BY a.file_identifier
        "
    );
    let result: Vec<File> = connection
        .prepare(query)?
        .into_iter()
        .map(|row_result| row_result.unwrap())
        .map(|row| {
            let path = row.read::<&str, _>("file_identifier").to_string();
            let hash = row.read::<&str, _>("hash").to_string();
            File { path, hash }
        })
        .collect();

    Ok(result)
}
