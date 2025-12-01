// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error, Lint};
use crate::slice_file::SliceFile;
use crate::slice_options::SliceOptions;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// A wrapper around a file path that implements Hash and Eq. This allows us to use a HashMap to store the path the user
/// supplied while using the canonicalized path as the key.
#[derive(Debug, Eq)]
struct FilePath {
    // The path that the user supplied
    path: String,
    // The canonicalized path
    canonicalized_path: PathBuf,
    // True for source files, false for reference files.
    is_source: bool,
}

impl FilePath {
    /// Creates a new [FilePath] from the given path. If the path does not exist, an [Error] is returned.
    pub fn try_create(path: &str, is_source: bool) -> Result<Self, io::Error> {
        PathBuf::from(path).canonicalize().map(|canonicalized_path| Self {
            path: path.to_owned(),
            canonicalized_path,
            is_source,
        })
    }
}

impl PartialEq for FilePath {
    fn eq(&self, other: &Self) -> bool {
        self.canonicalized_path == other.canonicalized_path
    }
}

/// This function takes a `Vec<FilePath>` and returns it, after removing any duplicate elements.
/// A lint violation is reported for each duplicate element.
fn remove_duplicate_file_paths(file_paths: Vec<FilePath>, diagnostics: &mut Diagnostics) -> Vec<FilePath> {
    let mut deduped_file_paths = Vec::with_capacity(file_paths.len());
    for file_path in file_paths {
        if deduped_file_paths.contains(&file_path) {
            let lint = Lint::DuplicateFile { path: file_path.path };
            Diagnostic::new(lint).push_into(diagnostics);
        } else {
            deduped_file_paths.push(file_path);
        }
    }
    deduped_file_paths
}

pub fn resolve_files_from(options: &SliceOptions, diagnostics: &mut Diagnostics) -> Vec<SliceFile> {
    let mut file_paths = Vec::new();

    // Add any source files to the list of file paths, after removing duplicates.
    let source_files = find_slice_files(&options.sources, true, diagnostics);
    file_paths.extend(remove_duplicate_file_paths(source_files, diagnostics));

    // Add any reference files to the list of file paths, after removing duplicates. We omit reference files that have
    // already been included as source files; we don't emit a warning for them, we just silently omit them. It's
    // important to do this after the source files, to ensure source files are given 'priority' over reference files.
    let reference_files = find_slice_files(&options.references, false, diagnostics);
    for reference_file in remove_duplicate_file_paths(reference_files, diagnostics) {
        if !file_paths.contains(&reference_file) {
            file_paths.push(reference_file);
        }
    }

    // Iterate through the discovered files and try to read them into Strings.
    // Report an error if it fails, otherwise create a new `SliceFile` to hold the data.
    let mut files = Vec::new();
    for file_path in file_paths {
        match fs::read_to_string(&file_path.path) {
            Ok(raw_text) => files.push(SliceFile::new(file_path.path, raw_text, file_path.is_source)),
            Err(error) => Diagnostic::new(Error::IO {
                action: "read",
                path: file_path.path,
                error,
            })
            .push_into(diagnostics),
        }
    }
    files
}

fn find_slice_files(paths: &[String], are_source_files: bool, diagnostics: &mut Diagnostics) -> Vec<FilePath> {
    // Directories can only be passed as references.
    let allow_directories = !are_source_files;

    let mut slice_paths = Vec::new();
    for path in paths {
        let path_buf = PathBuf::from(path);

        // If the path does not exist, report an error and continue.
        if !path_buf.exists() {
            Diagnostic::new(Error::IO {
                action: "read",
                path: path.to_owned(),
                error: io::ErrorKind::NotFound.into(),
            })
            .push_into(diagnostics);
            continue;
        }

        // If the path is a file but is not a Slice file, report an error and continue.
        if path_buf.is_file() && !is_slice_file(&path_buf) {
            // If the path is a file, check if it is a slice file.
            // TODO: It would be better to use `io::ErrorKind::InvalidFilename`, however it is an unstable feature.
            let io_error = io::Error::other("Slice files must end with a '.slice' extension");
            Diagnostic::new(Error::IO {
                action: "read",
                path: path.to_owned(),
                error: io_error,
            })
            .push_into(diagnostics);
            continue;
        }

        // If the path is a directory and directories are not allowed, report an error and continue.
        if path_buf.is_dir() && !allow_directories {
            // If the path is a file, check if it is a slice file.
            // TODO: It would be better to use `io::ErrorKind::InvalidFilename`, however it is an unstable feature.
            let io_error = io::Error::other("Expected a Slice file but found a directory.");
            Diagnostic::new(Error::IO {
                action: "read",
                path: path.to_owned(),
                error: io_error,
            })
            .push_into(diagnostics);
            continue;
        }

        slice_paths.extend(find_slice_files_in_path(path_buf, diagnostics));
    }

    slice_paths
        .into_iter()
        .map(|path| path.display().to_string())
        .filter_map(|path| match FilePath::try_create(&path, are_source_files) {
            Ok(file_path) => Some(file_path),
            Err(error) => {
                Diagnostic::new(Error::IO {
                    action: "read",
                    path,
                    error,
                })
                .push_into(diagnostics);
                None
            }
        })
        .collect()
}

fn find_slice_files_in_path(path: PathBuf, diagnostics: &mut Diagnostics) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if path.is_dir() {
        // Recurse into the directory.
        match find_slice_files_in_directory(&path, diagnostics) {
            Ok(child_paths) => paths.extend(child_paths),
            Err(error) => Diagnostic::new(Error::IO {
                action: "read",
                path: path.display().to_string(),
                error,
            })
            .push_into(diagnostics),
        }
    } else if path.is_file() && is_slice_file(&path) {
        // Add the file to the list of paths.
        paths.push(path);
    }
    // else we ignore the path

    paths
}

fn find_slice_files_in_directory(path: &Path, diagnostics: &mut Diagnostics) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let dir = path.read_dir()?;

    // Iterate though the directory and recurse into any subdirectories.
    for child in dir {
        match child {
            Ok(child) => paths.extend(find_slice_files_in_path(child.path(), diagnostics)),
            Err(error) => {
                // If we cannot read the directory entry, report an error and continue.
                Diagnostic::new(Error::IO {
                    action: "read",
                    path: path.display().to_string(),
                    error,
                })
                .push_into(diagnostics);
                continue;
            }
        }
    }
    Ok(paths)
}

/// Returns true if the path has the 'slice' extension.
fn is_slice_file(path: &Path) -> bool {
    path.extension().filter(|ext| ext.to_str() == Some("slice")).is_some()
}
