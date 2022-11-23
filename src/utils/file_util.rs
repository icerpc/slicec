// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::SliceOptions;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::slice_file::SliceFile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

pub fn get_files_from_options(options: &SliceOptions, diagnostic_reporter: &mut DiagnosticReporter) -> Vec<SliceFile> {
    // Create a map of all the Slice files with entries like: (absolute_path, is_source).
    // HashMap protects against files being passed twice (as reference and source).
    // It's important to add sources AFTER references, so sources overwrite references and not vice versa.
    let mut file_paths = HashMap::new();
    file_paths.extend(find_slice_files(&options.references).into_iter().map(|f| (f, false)));
    file_paths.extend(find_slice_files(&options.sources).into_iter().map(|f| (f, true)));

    // Iterate through the discovered files and try to read them into Strings.
    // Report an error if it fails, otherwise create a new `SliceFile` to hold the data.
    let mut files = Vec::new();
    for (file_path, is_source) in file_paths {
        match fs::read_to_string(&file_path) {
            Ok(raw_text) => files.push(SliceFile::new_unparsed(file_path, raw_text, is_source)),
            Err(err) => diagnostic_reporter.report_error(Error::new(ErrorKind::IO(err), None)),
        }
    }
    files
}

fn find_slice_files(paths: &[String]) -> Vec<String> {
    let mut slice_paths = Vec::new();
    for path in paths {
        match find_slice_files_in_path(PathBuf::from(path)) {
            Ok(child_paths) => slice_paths.extend(child_paths),
            Err(err) => eprintln!("failed to read file '{}': {}", path, err),
        }
    }

    slice_paths
        .into_iter()
        .map(|path| path.to_str().unwrap().to_owned())
        .collect()
}

fn find_slice_files_in_path(path: PathBuf) -> io::Result<Vec<PathBuf>> {
    // If the path is a directory, recursively search it for more slice files.
    if fs::metadata(&path)?.is_dir() {
        find_slice_files_in_directory(path.read_dir()?)
    }
    // If the path is not a directory, check if it ends with 'slice'.
    else if path.extension().filter(|ext| ext.to_str() == Some("slice")).is_some() {
        Ok(vec![path])
    } else {
        Ok(vec![])
    }
}

fn find_slice_files_in_directory(dir: fs::ReadDir) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for child in dir {
        let child_path = child?.path();
        match find_slice_files_in_path(child_path.clone()) {
            Ok(child_paths) => paths.extend(child_paths),
            Err(err) => eprintln!("failed to read file '{}': {}", child_path.display(), err),
        }
    }
    Ok(paths)
}
