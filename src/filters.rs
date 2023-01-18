use crate::file_manager::File;
use crate::params::Params;

pub fn is_file_gt_minsize(app_opts: &Params, file: &File) -> bool {
    match app_opts.get_minsize() {
        Some(msize) => match file.size {
            Some(fsize) => fsize >= msize,
            None => true,
        },
        None => true,
    }
}
