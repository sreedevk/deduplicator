#![allow(unused)]
use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::{fs, path::Path, time::Duration};

use globwalk::{GlobWalker, GlobWalkerBuilder};

pub struct Scanner {
    pub directory: Box<Path>,
    pub app_args: Arc<Params>,
    pub filetypes: Option<String>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub follow_links: bool,
    pub progress: bool,
}

impl Scanner {
    pub fn new(app_args: Arc<Params>) -> Result<Self> {
        Ok(Self {
            directory: app_args.get_directory()?.into_boxed_path(),
            filetypes: app_args.get_types(),
            min_depth: app_args.min_depth,
            max_depth: app_args.max_depth,
            min_size: app_args.get_min_size(),
            follow_links: app_args.follow_links,
            progress: app_args.progress,
            app_args,
        })
    }

    fn scan_patterns(&self) -> Result<String> {
        Ok(match &self.filetypes {
            Some(ftypes) => format!("**/*{{{ftypes}}}"),
            None => "**/*".to_string(),
        })
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
            self.directory.clone(),
            &[self.scan_patterns()?],
        ))
        .and_then(|walker| self.attach_walker_min_depth(walker))
        .and_then(|walker| self.attach_walker_max_depth(walker))
        .and_then(|walker| self.attach_link_opts(walker))?;

        Ok(walker.build()?)
    }

    pub fn scan(
        &self,
        files: Arc<Mutex<Vec<FileInfo>>>,
        progress_bar_box: Arc<MultiProgress>,
    ) -> Result<()> {
        let progress_bar = match self.progress {
            true => progress_bar_box.add(ProgressBar::new_spinner()),
            false => ProgressBar::hidden(),
        };

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
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
