#![allow(unused)]
use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::PathBuf, time::Duration};

use globwalk::{GlobWalker, GlobWalkerBuilder};

#[derive(Debug, Clone)]
pub struct Scanner {
    pub directory: Option<PathBuf>,
    pub filetypes: Option<String>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub follow_links: bool,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            directory: None,
            filetypes: None,
            min_depth: None,
            max_depth: None,
            min_size: None,
            follow_links: true,
        }
    }

    pub fn build(app_args: &Params) -> Result<Self> {
        let scan_directory = app_args.get_directory()?;
        Ok(Scanner::new())
            .map(|scanner| scanner.directory(scan_directory))
            .map(|scanner| match app_args.get_min_size() {
                Some(min_size) => scanner.min_size(min_size),
                None => scanner,
            })
            .map(|scanner| match app_args.get_types() {
                Some(ftypes) => scanner.filetypes(ftypes),
                None => scanner,
            })
            .map(|scanner| match app_args.min_depth {
                Some(min_depth) => scanner.min_depth(min_depth),
                None => scanner,
            })
            .map(|scanner| match app_args.max_depth {
                Some(max_depth) => scanner.max_depth(max_depth),
                None => scanner,
            })
    }

    pub fn min_size(&self, min_size: u64) -> Self {
        Self {
            min_size: Some(min_size),
            ..self.clone()
        }
    }

    pub fn min_depth(&self, min_depth: usize) -> Self {
        Self {
            min_depth: Some(min_depth),
            ..self.clone()
        }
    }

    pub fn max_depth(&self, max_depth: usize) -> Self {
        Self {
            max_depth: Some(max_depth),
            ..self.clone()
        }
    }

    pub fn directory(&self, dir: PathBuf) -> Self {
        Self {
            directory: Some(dir),
            ..self.clone()
        }
    }

    pub fn filetypes(&self, patterns: String) -> Self {
        Self {
            filetypes: Some(patterns),
            ..self.clone()
        }
    }

    pub fn ignore_links(&self) -> Self {
        Self {
            follow_links: false,
            ..self.clone()
        }
    }

    pub fn follow_links(&self) -> Self {
        Self {
            follow_links: true,
            ..self.clone()
        }
    }

    fn scan_patterns(&self) -> Result<String> {
        Ok(match self.filetypes.clone() {
            Some(ftypes) => format!("**/*{{{ftypes}}}"),
            None => "**/*".to_string(),
        })
    }

    fn scan_dir(&self) -> Result<PathBuf> {
        let scan_dir = match self.directory.clone() {
            Some(path) => path,
            None => std::env::current_dir()?,
        };

        Ok(fs::canonicalize(scan_dir)?)
    }

    fn attach_link_opts(&self, walker: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        Ok(walker.follow_links(self.follow_links))
    }

    fn attach_walker_min_depth(&self, walker: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        match self.min_depth {
            Some(min_depth) => Ok(walker.min_depth(min_depth)),
            None => Ok(walker),
        }
    }

    fn attach_walker_max_depth(&self, walker: GlobWalkerBuilder) -> Result<GlobWalkerBuilder> {
        match self.max_depth {
            Some(max_depth) => Ok(walker.max_depth(max_depth)),
            None => Ok(walker),
        }
    }
    fn build_walker(&self) -> Result<GlobWalker> {
        let walker = Ok(GlobWalkerBuilder::from_patterns(
            self.scan_dir()?,
            &[self.scan_patterns()?],
        ))
        .and_then(|walker| self.attach_walker_min_depth(walker))
        .and_then(|walker| self.attach_walker_max_depth(walker))
        .and_then(|walker| self.attach_link_opts(walker))?;

        Ok(walker.build()?)
    }

    pub fn scan(&self) -> Result<Vec<FileInfo>> {
        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("paths mapped");
        let min_size = self.min_size.unwrap_or_default();

        let results = self
            .build_walker()?
            .filter_map(Result::ok)
            .map(|entity| entity.into_path())
            .inspect(|_path| progress_bar.inc(1))
            .filter(|path| path.is_file())
            .map(FileInfo::new)
            .filter_map(Result::ok)
            .filter(|file| file.size > min_size)
            .collect::<Vec<FileInfo>>();

        progress_bar.finish_with_message("paths mapped");

        Ok(results)
    }
}
