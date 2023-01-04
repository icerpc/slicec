// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::SliceOptions;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::slice_file::SliceFile;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
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
    #[allow(clippy::result_large_err)]
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

    file_paths.extend(find_slice_files(&options.references, false, diagnostic_reporter));
    file_paths.extend(find_slice_files(&options.sources, true, diagnostic_reporter));

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
    source_files: bool,
    diagnostic_reporter: &mut DiagnosticReporter,
) -> Vec<(FilePath, bool)> {
    let mut slice_paths = Vec::new();
    for path in paths {
        match find_slice_files_in_path(PathBuf::from(path), diagnostic_reporter) {
            Ok(child_paths) => slice_paths.extend(child_paths),
            Err(error) => Error::new(ErrorKind::IO {
                action: "read",
                path: path.to_owned(),
                error,
            })
            .report(diagnostic_reporter),
        }
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
        .map(|f| (f, source_files))
        .collect()
}

fn find_slice_files_in_path(path: PathBuf, diagnostic_reporter: &mut DiagnosticReporter) -> io::Result<Vec<PathBuf>> {
    // If the path is a directory, recursively search it for more slice files.
    if fs::metadata(&path)?.is_dir() {
        find_slice_files_in_directory(path.read_dir()?, diagnostic_reporter)
    }
    // If the path is not a directory, check if it ends with 'slice'.
    else if path.extension().filter(|ext| ext.to_str() == Some("slice")).is_some() {
        Ok(vec![path])
    } else {
        Ok(vec![])
    }
}

fn find_slice_files_in_directory(
    dir: fs::ReadDir,
    diagnostic_reporter: &mut DiagnosticReporter,
) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for child in dir {
        let child_path = child?.path();
        match find_slice_files_in_path(child_path.clone(), diagnostic_reporter) {
            Ok(child_paths) => paths.extend(child_paths),
            Err(error) => Error::new(ErrorKind::IO {
                action: "read",
                path: child_path.display().to_string(),
                error,
            })
            .report(diagnostic_reporter),
        }
    }
    Ok(paths)
}
