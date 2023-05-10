//!
//! Utilities for directories and files within lookup and handling.
//!
use std::path::Path;
use std::path::PathBuf;

use crate::InitError;


/// Collect all files that match the given path to a backlog, and its adjacent chunks. Returns an
/// empty vector if there is no such main file going by the provided path.
pub fn find_files(path: &Path) -> Result<Vec<PathBuf>, InitError>
{
    let mut files = Vec::new();

    let stem = path.file_stem()
        .ok_or_else(|| InitError::NoStem {path: path.to_owned()})?
        .to_string_lossy()
        .to_string();

    let parent = path.parent()
        .ok_or_else(|| InitError::NoParent {path: path.to_owned()})?;

    let entries = std::fs::read_dir(parent)
        .map_err(|e| InitError::DirReadError {path: parent.to_owned(), source: e})?;

    for entry in entries
    {
        let entry = entry
            .map_err(|e| InitError::Unknown { path: path.to_owned(), source: e })?;

        let entry_path = entry.path();

        if let (Some(s), Some(e)) = (entry_path.file_stem(), entry_path.extension())
        {
            let entry_stem = s.to_string_lossy()
                .to_string();

            let entry_ext = e.to_string_lossy()
                .to_string();

            if entry_ext == "bkl" && entry_stem == stem {
                files.push(entry_path);
            }
        }
    }

    Ok(files)
}
