// Copyright (c) ZeroC, Inc. All rights reserved.
// TODO most of this module was just copy & pasted from the original implementation so that people
// can start using the newer implementation sooner.

mod comments;
mod cycle_detection;
mod encoding_patcher;
mod parent_patcher;
mod preprocessor;
mod slice;
mod type_patcher;

use crate::ast::Ast;
use crate::command_line::SliceOptions;
use crate::error::{Error, ErrorReporter};
use crate::slice_file::SliceFile;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

// NOTE! it is NOT safe to call any methods on any of the slice entities during parsing.
// Slice entities are NOT considered fully constructed until AFTER parsing is finished (including patching).
// Accessing ANY data, or calling ANY methods before this point may result in panics or undefined behavior.

// TODO This module is a mess.

pub fn parse_files(ast: &mut Ast, options: &SliceOptions) -> Result<HashMap<String, SliceFile>, Error> {
    let mut error_reporter = ErrorReporter::default();
    let mut parser = slice::SliceParser { error_reporter: &mut error_reporter };

    let source_files = find_slice_files(&options.sources);
    let mut reference_files = find_slice_files(&options.references);
    // Remove duplicate reference files, or files that are already being parsed as source.
    // This ensures that a file isn't parsed twice, which would cause redefinition errors.
    reference_files.retain(|file| !source_files.contains(file));
    reference_files.sort();
    reference_files.dedup();

    let mut slice_files = HashMap::new();

    for path in source_files {
        if let Some(slice_file) = parser.try_parse_file(&path, true, ast) {
            slice_files.insert(path.to_owned(), slice_file);
        }
    }
    for path in reference_files {
        if let Some(slice_file) = parser.try_parse_file(&path, false, ast) {
            slice_files.insert(path.to_owned(), slice_file);
        }
    }

    parent_patcher::patch_parents(ast);
    crate::handle_errors(options.warn_as_error, &slice_files, &mut error_reporter)?;
    type_patcher::patch_types(ast, &mut error_reporter);
    crate::handle_errors(options.warn_as_error, &slice_files, &mut error_reporter)?;
    cycle_detection::detect_cycles(&slice_files, &mut error_reporter);
    crate::handle_errors(options.warn_as_error, &slice_files, &mut error_reporter)?;
    encoding_patcher::patch_encodings(&slice_files, ast, &mut error_reporter);

    Ok(slice_files)
}

pub fn parse_string(input: &str, error_reporter: &mut ErrorReporter) -> Result<(Ast, HashMap<String,SliceFile>), Error> {
    let mut parser = slice::SliceParser { error_reporter };
    let mut ast = Ast::new();

    let identifier = "in-memory-file";

    let slice_file = parser.parse_string(identifier, input, &mut ast)?;

    let slice_files = HashMap::from([(identifier.to_owned(), slice_file)]);

    // Patch the AST.
    parent_patcher::patch_parents(&mut ast);
    type_patcher::patch_types(&mut ast, error_reporter);

    parent_patcher::patch_parents(&mut ast);
    type_patcher::patch_types(&mut ast, error_reporter);
    cycle_detection::detect_cycles(&slice_files, error_reporter);
    encoding_patcher::patch_encodings(&slice_files, &mut ast, error_reporter);

    Ok((ast, slice_files))
}


fn find_slice_files(paths: &[String]) -> Vec<String> {
    let mut slice_paths = Vec::new();
    for path in paths {
        match find_slice_files_in_path(PathBuf::from(path)) {
            Ok(child_paths) => slice_paths.extend(child_paths),
            Err(err) => eprintln!("failed to read file '{}': {}", path, err),
        }
    }

    let mut string_paths = slice_paths.into_iter()
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
