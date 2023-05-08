use crate::file_manager::File;
use crate::params::Params;

pub fn is_file_gt_min_size(app_opts: &Params, file: &File) -> bool {
    match app_opts.get_min_size() {
        Some(msize) => match file.size {
            Some(fsize) => fsize >= msize,
            None => true,
        },
        None => true,
    }
}
