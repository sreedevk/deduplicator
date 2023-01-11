use crate::database::File;
use anyhow::Result;

pub fn delete_files(files: Vec<File>) -> Result<()> {
    println!("{:?} DELETED", files);
    Ok(())
}
