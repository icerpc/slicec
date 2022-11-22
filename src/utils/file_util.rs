// Copyright (c) ZeroC, Inc. All rights reserved.

use std::path::PathBuf;
use std::{fs, io};

pub fn find_slice_files(paths: &[String]) -> Vec<String> {
    let mut slice_paths = Vec::new();
    for path in paths {
        match find_slice_files_in_path(PathBuf::from(path)) {
            Ok(child_paths) => slice_paths.extend(child_paths),
            Err(err) => eprintln!("failed to read file '{}': {}", path, err),
        }
    }

    let mut string_paths = slice_paths
        .into_iter()
        .map(|path| path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    string_paths.sort();
    string_paths.dedup();
    string_paths
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
