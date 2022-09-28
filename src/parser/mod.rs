// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO most of this module was just copy & pasted from the original implementation so that people
// can start using the newer implementation sooner.

mod comments;
mod cycle_detection;
mod preprocessor;
mod slice;

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::attributes;
use crate::parse_result::{ParsedData, ParserResult};
use crate::slice_file::SliceFile;

use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

// NOTE! it is NOT safe to call any methods on any of the slice entities during parsing.
// Slice entities are NOT considered fully constructed until AFTER parsing is finished (including
// patching). Accessing ANY data, or calling ANY methods before this point may result in panics or
// undefined behavior.

// TODO This module is a mess.

pub fn parse_files(options: &SliceOptions) -> ParserResult {
    let mut ast = Ast::create();
    let mut diagnostic_reporter = DiagnosticReporter::new(options);

    let mut parser = slice::SliceParser {
        diagnostic_reporter: &mut diagnostic_reporter,
    };

    let source_files = find_slice_files(&options.sources);
    let mut reference_files = find_slice_files(&options.references);
    // Remove duplicate reference files, or files that are already being parsed as source.
    // This ensures that a file isn't parsed twice, which would cause redefinition errors.
    reference_files.retain(|file| !source_files.contains(file));
    reference_files.sort();
    reference_files.dedup();

    let mut slice_files = HashMap::new();

    for path in source_files {
        if let Some(slice_file) = parser.try_parse_file(&path, true, &mut ast) {
            slice_files.insert(path.to_owned(), slice_file);
        }
    }

    for path in reference_files {
        if let Some(slice_file) = parser.try_parse_file(&path, false, &mut ast) {
            slice_files.insert(path.to_owned(), slice_file);
        }
    }

    // Update the diagnostic reporter with the slice files that contain the file level ignore_warnings attribute.
    diagnostic_reporter.file_level_ignored_warnings = file_ignored_warnings_map(&slice_files);
    let parsed_data = ParsedData {
        ast,
        files: slice_files,
        diagnostic_reporter,
    };

    patch_ast(parsed_data)
}

pub fn parse_strings(inputs: &[&str], options: Option<SliceOptions>) -> ParserResult {
    let mut ast = Ast::create();
    let mut slice_options = options.unwrap_or(SliceOptions {
        sources: vec![],
        references: vec![],
        warn_as_error: true,
        disable_color: false,
        diagnostic_format: DiagnosticFormat::Human,
        validate: false,
        output_dir: None,
    });
    slice_options.warn_as_error = true;
    let mut diagnostic_reporter = DiagnosticReporter::new(&slice_options);
    let mut parser = slice::SliceParser {
        diagnostic_reporter: &mut diagnostic_reporter,
    };

    let mut slice_files = HashMap::new();

    for (i, input) in inputs.iter().enumerate() {
        if let Some(slice_file) = parser.try_parse_string(&format!("string-{}", i), input, &mut ast) {
            slice_files.insert(slice_file.filename.clone(), slice_file);
        }
    }

    // Update the diagnostic reporter with the slice files that contain the file level ignore_warnings attribute.
    diagnostic_reporter.file_level_ignored_warnings = file_ignored_warnings_map(&slice_files);
    let parsed_data = ParsedData {
        ast,
        files: slice_files,
        diagnostic_reporter,
    };

    patch_ast(parsed_data)
}

fn patch_ast(mut parsed_data: ParsedData) -> ParserResult {
    // TODO integrate this better with ParsedData in the future.
    if !parsed_data.has_errors() {
        unsafe {
            parsed_data = crate::ast::patch_ast(parsed_data)?;
        }
    }

    // TODO move this to a validator now that the patchers can handle traversing cycles on their own.
    if !parsed_data.has_errors() {
        cycle_detection::detect_cycles(&parsed_data.files, &mut parsed_data.diagnostic_reporter);
    }

    parsed_data.into()
}

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

// Returns a HashMap where the keys are the relative paths of the .slice files that have the file level
// `ignore_warnings` attribute and the values are the arguments of the attribute.
fn file_ignored_warnings_map(files: &HashMap<String, SliceFile>) -> HashMap<String, Vec<String>> {
    files
        .iter()
        .filter_map(|(path, file)| {
            file.attributes
                .iter()
                .find(|attr| attr.directive == attributes::IGNORE_WARNINGS)
                .map(|attr| attr.arguments.clone())
                .map(|ignored_warnings| (path.to_owned(), ignored_warnings))
        })
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
