use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub trait FileScanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf>;
}

#[derive(Default)]
pub struct Mp3FileScanner;

impl FileScanner for Mp3FileScanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"))
            })
            .collect();

        files.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        files
    }
}
