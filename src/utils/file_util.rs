// Copyright (c) ZeroC, Inc.

use crate::command_line::SliceOptions;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind, Warning, WarningKind};
use crate::slice_file::SliceFile;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
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
}

impl TryFrom<&String> for FilePath {
    type Error = io::Error;

    /// Creates a new [FilePath] from the given path. If the path does not exist, an [Error] is returned.
    fn try_from(path: &String) -> Result<Self, Self::Error> {
        PathBuf::from(&path).canonicalize().map(|canonicalized_path| Self {
            path: path.clone(),
            canonicalized_path,
        })
    }
}

impl Hash for FilePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.canonicalized_path.hash(state);
    }
}

impl PartialEq for FilePath {
    fn eq(&self, other: &Self) -> bool {
        self.canonicalized_path == other.canonicalized_path
    }
}

pub fn resolve_files_from(options: &SliceOptions, diagnostic_reporter: &mut DiagnosticReporter) -> Vec<SliceFile> {
    // Create a map of all the Slice files with entries like: (absolute_path, is_source).
    // HashMap protects against files being passed twice (as reference and source).
    // It's important to add sources AFTER references, so sources overwrite references and not vice versa.
    let mut file_paths = HashMap::new();

    let reference_files = find_slice_files(&options.references, true, diagnostic_reporter);

    // Any duplicate reference files will be reported as a warning.
    for reference_file in reference_files {
        let path = reference_file.path.clone();
        if file_paths.insert(reference_file, false).is_some() {
            Warning::new(WarningKind::DuplicateFile { path }).report(diagnostic_reporter);
        }
    }

    let source_files = find_slice_files(&options.sources, false, diagnostic_reporter);

    // Any duplicate source files (that duplicate another source file, not a reference file) will be reported as
    // a warning.
    for source_file in source_files {
        let path = source_file.path.clone();
        // Insert will return replace and return the previous value if the key already exists.
        // We use this to allow replacing references with sources.
        if let Some(is_source) = file_paths.insert(source_file, true) {
            // Only report an error if the file was previously a source file.
            if is_source {
                Warning::new(WarningKind::DuplicateFile { path }).report(diagnostic_reporter);
            }
        }
    }

    // Iterate through the discovered files and try to read them into Strings.
    // Report an error if it fails, otherwise create a new `SliceFile` to hold the data.
    let mut files = Vec::new();
    for (file_path, is_source) in file_paths {
        match fs::read_to_string(&file_path.path) {
            Ok(raw_text) => files.push(SliceFile::new(file_path.path, raw_text, is_source)),
            Err(error) => Error::new(ErrorKind::IO {
                action: "read",
                path: file_path.path,
                error,
            })
            .report(diagnostic_reporter),
        }
    }

    files
}

fn find_slice_files(
    paths: &[String],
    allow_directories: bool,
    diagnostic_reporter: &mut DiagnosticReporter,
) -> Vec<FilePath> {
    let mut slice_paths = Vec::new();
    for path in paths {
        let path_buf = PathBuf::from(path);

        // If the path does not exist, report an error and continue to the next path.
        if !path_buf.exists() {
            // If the path does not exist, report an error and continue.
            Error::new(ErrorKind::IO {
                action: "read",
                path: path.to_owned(),
                error: io::ErrorKind::NotFound.into(),
            })
            .report(diagnostic_reporter);
            continue;
        }

        // If the path is a file but is not a Slice file, report an error and continue.
        if path_buf.is_file() && !is_slice_file(&path_buf) {
            // If the path is a file, check if it is a slice file.
            // TODO: It would be better to use `io::ErrorKind::InvalidFilename`, however it is an unstable feature.
            let io_error = io::Error::new(io::ErrorKind::Other, "Slice files must end with a '.slice' extension");
            Error::new(ErrorKind::IO {
                action: "read",
                path: path.to_owned(),
                error: io_error,
            })
            .report(diagnostic_reporter);
            continue;
        }

        // If the path is a directory and directories are not allowed, report an error and continue.
        if path_buf.is_dir() && !allow_directories {
            // If the path is a file, check if it is a slice file.
            // TODO: It would be better to use `io::ErrorKind::InvalidFilename`, however it is an unstable feature.
            let io_error = io::Error::new(io::ErrorKind::Other, "Excepted a Slice file but found a directory.");
            Error::new(ErrorKind::IO {
                action: "read",
                path: path.to_owned(),
                error: io_error,
            })
            .report(diagnostic_reporter);
            continue;
        }

        slice_paths.extend(find_slice_files_in_path(path_buf, diagnostic_reporter));
    }

    slice_paths
        .into_iter()
        .map(|path| path.display().to_string())
        .filter_map(|path| match FilePath::try_from(&path) {
            Ok(file_path) => Some(file_path),
            Err(error) => {
                Error::new(ErrorKind::IO {
                    action: "read",
                    path,
                    error,
                })
                .report(diagnostic_reporter);
                None
            }
        })
        .collect()
}

fn find_slice_files_in_path(path: PathBuf, diagnostic_reporter: &mut DiagnosticReporter) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if path.is_dir() {
        // Recurse into the directory.
        match find_slice_files_in_directory(&path, diagnostic_reporter) {
            Ok(child_paths) => paths.extend(child_paths),
            Err(error) => Error::new(ErrorKind::IO {
                action: "read",
                path: path.display().to_string(),
                error,
            })
            .report(diagnostic_reporter),
        }
    } else if path.is_file() && is_slice_file(&path) {
        // Add the file to the list of paths.
        paths.push(path.to_path_buf());
    }
    // else we ignore the path

    paths
}

fn find_slice_files_in_directory(
    path: &Path,
    diagnostic_reporter: &mut DiagnosticReporter,
) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let dir = path.read_dir()?;

    // Iterate though the directory and recurse into any subdirectories.
    for child in dir {
        match child {
            Ok(child) => paths.extend(find_slice_files_in_path(child.path(), diagnostic_reporter)),
            Err(error) => {
                // If we cannot read the directory entry, report an error and continue.
                Error::new(ErrorKind::IO {
                    action: "read",
                    path: path.display().to_string(),
                    error,
                })
                .report(diagnostic_reporter);
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
