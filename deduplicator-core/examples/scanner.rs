use std::path::PathBuf;

use anyhow::Result;
use deduplicator_core::processor::Processor;
use deduplicator_core::scanner::Scanner;

fn main() -> Result<()> {
    let scan_results: Vec<_> = Scanner::new()
        .directory(PathBuf::from("/home/sreedev/Data/books/"))
        .ignore_links().scan(None)?;

    let processor = Processor::new(scan_results);
    let final_processor = processor.sizewise()?.hashwise()?;

    dbg!(final_processor);


    Ok(())
}
