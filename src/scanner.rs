use crate::{fileinfo::FileInfo, params::Params};
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::{path::Path, time::Duration};

use globwalk::{GlobWalker, GlobWalkerBuilder};

pub struct Scanner {
    pub directory: Box<Path>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub include_types: Option<String>,
    pub exclude_types: Option<String>,
    pub min_size: Option<u64>,
    pub follow_links: bool,
    pub progress: bool,
}

impl Scanner {
    pub fn new(app_args: Arc<Params>) -> Result<Self> {
        Ok(Self {
            directory: app_args.get_directory()?.into_boxed_path(),
            include_types: app_args.types.clone(),
            exclude_types: app_args.exclude_types.clone(),
            min_depth: app_args.min_depth,
            max_depth: app_args.max_depth,
            min_size: app_args.get_min_size(),
            follow_links: app_args.follow_links,
            progress: app_args.progress,
        })
    }

    fn scan_patterns(&self) -> Result<Vec<String>> {
        let include_types = match &self.include_types {
            Some(ftypes) => Some(format!("**/*.{{{ftypes}}}")),
            None => Some("**/*".to_string()),
        };

        let exclude_types = self
            .exclude_types
            .as_ref()
            .map(|ftypes| format!("!**/*.{{{ftypes}}}"));

        Ok(vec![include_types, exclude_types]
            .into_iter()
            .flatten()
            .collect())
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
            &self.scan_patterns()?,
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
        let min_size = self.min_size.unwrap_or(0);

        self.build_walker()?
            .filter_map(Result::ok)
            .map(|entity| entity.into_path())
            .inspect(|_path| progress_bar.inc(1))
            .filter(|path| path.is_file())
            .map(FileInfo::new)
            .filter_map(Result::ok)
            .filter(|file| file.size >= min_size)
            .for_each(|file| {
                let mut flock = files.lock().unwrap();
                flock.push(file);
            });

        progress_bar.finish_with_message("paths mapped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fileinfo::FileInfo;
    use crate::params::Params;
    use std::fs::File;
    use std::sync::{Arc, Mutex};

    use super::Scanner;
    use indicatif::MultiProgress;
    use tempfile::TempDir;

    #[test]
    fn ensure_file_include_type_filter_includes_expected_file_types() {
        let root =
            TempDir::with_prefix("deduplicator_test_root").expect("unable to create tempdir");
        [
            "this-is-a-js-file.js",
            "this-is-a-css-file.css",
            "this-is-a-csv-file.csv",
            "this-is-a-rust-file.rs",
        ]
        .iter()
        .for_each(|path| {
            File::create_new(root.path().join(path)).unwrap_or_else(|_| {
                panic!("unable to create file {path}");
            });
        });

        let params = Params {
            types: Some(String::from("js,csv")),
            dir: Some(root.path().into()),
            ..Default::default()
        };

        let progress = Arc::new(MultiProgress::new());
        let scanlist = Arc::new(Mutex::<Vec<FileInfo>>::new(vec![]));
        let scanner = Scanner::new(Arc::new(params)).expect("scanner initialization failed");

        scanner
            .scan(scanlist.clone(), progress)
            .expect("scanning failed.");

        let scan_list_mg = scanlist.lock().unwrap();

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-js-file.js").to_str().unwrap()));

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-csv-file.csv").to_str().unwrap()));

        assert!(scan_list_mg.iter().all(|f| f.path.to_str().unwrap()
            != root.path().join("this-is-a-css-file.css").to_str().unwrap()));

        assert!(scan_list_mg.iter().all(|f| f.path.to_str().unwrap()
            != root.path().join("this-is-a-rust-file.rs").to_str().unwrap()));
    }

    #[test]
    fn ensure_file_exclude_type_filter_excludes_expected_file_types() {
        let root =
            TempDir::with_prefix("deduplicator_test_root").expect("unable to create tempdir");
        [
            "this-is-a-js-file.js",
            "this-is-a-css-file.css",
            "this-is-a-csv-file.csv",
            "this-is-a-rust-file.rs",
        ]
        .iter()
        .for_each(|path| {
            File::create_new(root.path().join(path)).unwrap_or_else(|_| {
                panic!("unable to create file {path}");
            });
        });

        let params = Params {
            exclude_types: Some(String::from("js,csv")),
            dir: Some(root.path().into()),
            ..Default::default()
        };

        let progress = Arc::new(MultiProgress::new());
        let scanlist = Arc::new(Mutex::<Vec<FileInfo>>::new(vec![]));
        let scanner = Scanner::new(Arc::new(params)).expect("scanner initialization failed");

        scanner
            .scan(scanlist.clone(), progress)
            .expect("scanning failed.");

        let scan_list_mg = scanlist.lock().unwrap();

        assert!(scan_list_mg.iter().all(|f| f.path.to_str().unwrap()
            != root.path().join("this-is-a-js-file.js").to_str().unwrap()));

        assert!(scan_list_mg.iter().all(|f| f.path.to_str().unwrap()
            != root.path().join("this-is-a-csv-file.csv").to_str().unwrap()));

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-css-file.css").to_str().unwrap()));

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-rust-file.rs").to_str().unwrap()));
    }

    #[test]
    fn complex_file_type_params() {
        let root =
            TempDir::with_prefix("deduplicator_test_root").expect("unable to create tempdir");
        [
            "this-is-a-js-file.js",
            "this-is-a-css-file.css",
            "this-is-a-csv-file.csv",
            "this-is-a-rust-file.rs",
        ]
        .iter()
        .for_each(|path| {
            File::create_new(root.path().join(path)).unwrap_or_else(|_| {
                panic!("unable to create file {path}");
            });
        });

        let params = Params {
            types: Some(String::from("js,csv,rs")),
            exclude_types: Some(String::from("csv")),
            dir: Some(root.path().into()),
            ..Default::default()
        };

        let progress = Arc::new(MultiProgress::new());
        let scanlist = Arc::new(Mutex::<Vec<FileInfo>>::new(vec![]));
        let scanner = Scanner::new(Arc::new(params)).expect("scanner initialization failed");

        scanner
            .scan(scanlist.clone(), progress)
            .expect("scanning failed.");

        let scan_list_mg = scanlist.lock().unwrap();

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-js-file.js").to_str().unwrap()));

        assert!(scan_list_mg.iter().all(|f| f.path.to_str().unwrap()
            != root.path().join("this-is-a-csv-file.csv").to_str().unwrap()));

        assert!(scan_list_mg.iter().any(|f| f.path.to_str().unwrap()
            == root.path().join("this-is-a-rust-file.rs").to_str().unwrap()));
    }
}
