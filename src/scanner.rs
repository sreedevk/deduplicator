#![allow(unused)]
use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
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
        let mut scanner = Scanner::new();
        scanner.directory = Some(app_args.get_directory()?);

        if let Some(min_size) = app_args.get_min_size() {
            scanner.min_size = Some(min_size);
        }

        if app_args.get_types().is_some() {
            scanner.filetypes = app_args.get_types();
        }

        if app_args.min_depth.is_some() {
            scanner.min_depth = app_args.min_depth;
        }

        if app_args.max_depth.is_some() {
            scanner.max_depth = app_args.max_depth;
        }

        Ok(scanner)
    }

    pub fn filetypes(&mut self, patterns: String) {
        self.filetypes = Some(patterns);
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

    pub fn scan(&self, files: Arc<Mutex<Vec<FileInfo>>>) -> Result<()> {
        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("paths mapped");
        let min_size = self.min_size.unwrap_or_default();

        self.build_walker()?
            .filter_map(Result::ok)
            .map(|entity| entity.into_path())
            .inspect(|_path| progress_bar.inc(1))
            .filter(|path| path.is_file())
            .map(FileInfo::new)
            .filter_map(Result::ok)
            .filter(|file| file.size > min_size)
            .for_each(|file| {
                let mut flock = files.lock().unwrap();
                flock.push(file);
            });

        progress_bar.finish_with_message("paths mapped");
        Ok(())
    }
}
