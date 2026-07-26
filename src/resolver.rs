use std::cmp::Ordering;

use unicode_segmentation::UnicodeSegmentation;
use anyhow::{bail, Result};
use bytesize::ByteSize;

use crate::fileinfo::FileInfo;
use crate::formatter::Formatter;
use crate::params::Params;
use crate::pipeline::DedupReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum KeepStrategy {
    Newest,
    Oldest,
    First,
    Last,
    Shortest,
    Shallowest,
}

fn path_string(file: &FileInfo) -> String {
    file.path.to_string_lossy().into_owned()
}

fn path_len(file: &FileInfo) -> usize {
    file.path.to_string_lossy().graphemes(true).count()
}

fn depth(file: &FileInfo) -> usize {
    file.path.iter().count()
}

fn compare_keep(a: &FileInfo, b: &FileInfo, strategy: KeepStrategy) -> Ordering {
    match strategy {
        KeepStrategy::Newest => b
            .modified
            .cmp(&a.modified)
            .then_with(|| path_string(a).cmp(&path_string(b))),
        KeepStrategy::Oldest => a
            .modified
            .cmp(&b.modified)
            .then_with(|| path_string(a).cmp(&path_string(b))),
        KeepStrategy::First => path_string(a).cmp(&path_string(b)),
        KeepStrategy::Last => path_string(b).cmp(&path_string(a)),
        KeepStrategy::Shortest => path_len(a)
            .cmp(&path_len(b))
            .then_with(|| path_string(a).cmp(&path_string(b))),
        KeepStrategy::Shallowest => depth(a)
            .cmp(&depth(b))
            .then_with(|| path_len(a).cmp(&path_len(b)))
            .then_with(|| path_string(a).cmp(&path_string(b))),
    }
}

pub fn select_keeper(files: &[FileInfo], strategy: KeepStrategy) -> usize {
    files
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| compare_keep(a, b, strategy))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

pub fn run(
    report: &DedupReport,
    strategy: KeepStrategy,
    force: bool,
    params: &Params,
) -> Result<()> {
    if report.groups.is_empty() {
        println!("No duplicates found matching your search criteria.");
        return Ok(());
    }

    let group_count = report.groups.len();
    let mut victim_count: u64 = 0;
    let mut victim_bytes: u64 = 0;
    let mut freed_bytes: u64 = 0;
    let mut failures: u64 = 0;

    for group in &report.groups {
        let keeper = select_keeper(&group.files, strategy);

        if !force {
            println!(
                "KEEP    {}",
                Formatter::human_path(&group.files[keeper], params, report.max_path_len)
                    .unwrap_or_default()
            );
        }

        for (index, file) in group.files.iter().enumerate() {
            if index == keeper {
                continue;
            }

            victim_count += 1;
            victim_bytes += file.size;

            match force {
                false => println!(
                    "DELETE  {}  {}",
                    Formatter::human_path(file, params, report.max_path_len).unwrap_or_default(),
                    Formatter::human_filesize(file).unwrap_or_default()
                ),
                true => match std::fs::remove_file(&file.path) {
                    Ok(_) => {
                        freed_bytes += file.size;
                        println!("deleted {}", file.path.display());
                    }
                    Err(_) => {
                        failures += 1;
                        println!("FAILED  {}", file.path.display());
                    }
                },
            }
        }
    }

    match force {
        false => println!(
            "\n{group_count} groups, would free {}. Re-run with --force to delete.",
            ByteSize::b(victim_bytes)
        ),
        true => println!(
            "\ndeleted {} files, freed {} across {group_count} groups.",
            victim_count - failures,
            ByteSize::b(freed_bytes)
        ),
    }

    if failures > 0 {
        bail!("{failures} deletion(s) failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileinfo::FileInfo;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use crate::params::Params;
    use crate::pipeline::{DedupReport, DuplicateGroup};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn file(path: &str, mtime_secs: u64) -> FileInfo {
        FileInfo {
            path: PathBuf::from(path).into_boxed_path(),
            size: 0,
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(mtime_secs),
        }
    }

    #[test]
    fn newest_keeps_greatest_mtime() {
        let files = vec![file("/a/x", 10), file("/a/y", 30), file("/a/z", 20)];
        assert_eq!(select_keeper(&files, KeepStrategy::Newest), 1);
    }

    #[test]
    fn oldest_keeps_least_mtime() {
        let files = vec![file("/a/x", 10), file("/a/y", 30), file("/a/z", 20)];
        assert_eq!(select_keeper(&files, KeepStrategy::Oldest), 0);
    }

    #[test]
    fn first_keeps_lexicographically_smallest_path() {
        let files = vec![file("/a/y", 10), file("/a/x", 10), file("/a/z", 10)];
        assert_eq!(select_keeper(&files, KeepStrategy::First), 1);
    }

    #[test]
    fn last_keeps_lexicographically_greatest_path() {
        let files = vec![file("/a/y", 10), file("/a/x", 10), file("/a/z", 10)];
        assert_eq!(select_keeper(&files, KeepStrategy::Last), 2);
    }

    #[test]
    fn shortest_keeps_fewest_chars() {
        let files = vec![file("/aaa/bbb", 10), file("/a/b", 10), file("/aa/bb", 10)];
        assert_eq!(select_keeper(&files, KeepStrategy::Shortest), 1);
    }

    #[test]
    fn shallowest_keeps_fewest_components() {
        let files = vec![file("/a/b/c/d", 10), file("/a/b", 10), file("/a/b/c", 10)];
        assert_eq!(select_keeper(&files, KeepStrategy::Shallowest), 1);
    }

    #[test]
    fn newest_tiebreak_is_smallest_path_regardless_of_order() {
        let forward = vec![file("/a/y", 30), file("/a/x", 30)];
        let reversed = vec![file("/a/x", 30), file("/a/y", 30)];
        assert_eq!(
            forward[select_keeper(&forward, KeepStrategy::Newest)].path,
            reversed[select_keeper(&reversed, KeepStrategy::Newest)].path
        );
        assert_eq!(
            forward[select_keeper(&forward, KeepStrategy::Newest)]
                .path
                .to_string_lossy(),
            "/a/x"
        );
    }

    fn write_dup_report(root: &TempDir, names: &[&str]) -> DedupReport {
        let files = names
            .iter()
            .map(|name| {
                let path = root.path().join(name);
                let mut f = File::create_new(&path).unwrap();
                f.write_all(b"identical duplicate payload").unwrap();
                FileInfo::new(path).unwrap()
            })
            .collect::<Vec<_>>();

        DedupReport {
            groups: vec![DuplicateGroup { hash: 0, files }],
            max_path_len: 0,
        }
    }

    #[test]
    fn force_deletes_victims_and_keeps_the_keeper() {
        let root = TempDir::new().unwrap();
        let report = write_dup_report(&root, &["a.bin", "b.bin"]);
        let params = Params {
            dir: Some(root.path().into()),
            ..Default::default()
        };

        super::run(&report, KeepStrategy::First, true, &params).unwrap();

        assert!(root.path().join("a.bin").exists());
        assert!(!root.path().join("b.bin").exists());
    }

    #[test]
    fn dry_run_deletes_nothing() {
        let root = TempDir::new().unwrap();
        let report = write_dup_report(&root, &["a.bin", "b.bin"]);
        let params = Params {
            dir: Some(root.path().into()),
            ..Default::default()
        };

        super::run(&report, KeepStrategy::First, false, &params).unwrap();

        assert!(root.path().join("a.bin").exists());
        assert!(root.path().join("b.bin").exists());
    }

    #[test]
    fn force_reports_error_when_a_deletion_fails() {
        let root = TempDir::new().unwrap();
        let report = write_dup_report(&root, &["a.bin", "b.bin"]);
        let params = Params {
            dir: Some(root.path().into()),
            ..Default::default()
        };

        std::fs::remove_file(root.path().join("b.bin")).unwrap();

        let result = super::run(&report, KeepStrategy::First, true, &params);
        assert!(result.is_err());
    }

    #[test]
    fn empty_report_is_ok() {
        let params = Params::default();
        let report = DedupReport {
            groups: vec![],
            max_path_len: 0,
        };
        assert!(super::run(&report, KeepStrategy::First, false, &params).is_ok());
    }
}
