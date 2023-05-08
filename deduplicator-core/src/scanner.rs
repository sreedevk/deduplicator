#![allow(unused)]
use anyhow::Result;
use std::{fs, path::PathBuf};
use crate::fileinfo::FileInfo;

use globwalk::{GlobWalker, GlobWalkerBuilder};

#[derive(Debug, Clone)]
pub struct Scanner {
    pub directory: Option<PathBuf>,
    pub filetypes: Option<String>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub follow_links: bool,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            directory: None,
            filetypes: None,
            min_depth: None,
            max_depth: None,
            follow_links: true,
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
        Ok(self
            .build_walker()?
            .filter_map(Result::ok)
            .map(|entity| entity.into_path())
            .filter(|path| path.is_file())
            .map(|file| FileInfo::new(file))
            .filter_map(Result::ok)
            .collect::<Vec<FileInfo>>())
    }
}
