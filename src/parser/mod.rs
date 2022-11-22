// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod comments;

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::grammar::attributes;
use crate::slice_file::SliceFile;
use crate::utils::file_util;

use std::collections::{HashMap, HashSet};

// NOTE! it is NOT safe to call any methods on any of the slice entities during parsing.
// Slice entities are NOT considered fully constructed until AFTER parsing is finished (including
// patching). Accessing ANY data, or calling ANY methods before this point may result in panics or
// undefined behavior.

pub fn parse_files(options: &SliceOptions) -> CompilationResult {
    let mut data = CompilationData {
        ast: Ast::create(),
        diagnostic_reporter: DiagnosticReporter::new(options),
        files: HashMap::new(),
    };

    let source_files = file_util::find_slice_files(&options.sources);
    let mut reference_files = file_util::find_slice_files(&options.references);
    // Remove duplicate reference files, or files that are already being parsed as source.
    // This ensures that a file isn't parsed twice, which would cause redefinition errors.
    reference_files.retain(|file| !source_files.contains(file));
    reference_files.sort();
    reference_files.dedup();

    for path in source_files {
        if let Some(slice_file) = try_parse_file(&path, true, &mut data) {
            data.files.insert(path.to_owned(), slice_file);
        }
    }

    for path in reference_files {
        if let Some(slice_file) = try_parse_file(&path, false, &mut data) {
            data.files.insert(path.to_owned(), slice_file);
        }
    }

    // Update the diagnostic reporter with the slice files that contain the file level ignoreWarnings attribute.
    data.diagnostic_reporter.file_level_ignored_warnings = file_ignored_warnings_map(&data.files);

    patch_ast(data)
}

fn try_parse_file(file: &str, is_source: bool, data: &mut CompilationData) -> Option<SliceFile> {
    match std::fs::read_to_string(file) {
        Ok(raw_text) => {
            // The parser emits errors through `DiagnosticReporter` on it's own, so we don't need to handle them.
            try_parse_string(file, &raw_text, is_source, data).ok()
        }
        Err(err) => {
            data.diagnostic_reporter
                .report_error(Error::new(ErrorKind::IO(err), None));
            None
        }
    }
}

pub fn parse_strings(inputs: &[&str], options: Option<SliceOptions>) -> CompilationResult {
    let slice_options = options.unwrap_or(SliceOptions {
        sources: vec![],
        references: vec![],
        warn_as_error: true,
        disable_color: false,
        diagnostic_format: DiagnosticFormat::Human,
        validate: false,
        output_dir: None,
    });

    let mut data = CompilationData {
        ast: Ast::create(),
        diagnostic_reporter: DiagnosticReporter::new(&slice_options),
        files: HashMap::new(),
    };

    for (i, input) in inputs.iter().enumerate() {
        let name = format!("string-{}", i);
        if let Ok(slice_file) = try_parse_string(&name, input, false, &mut data) {
            data.files.insert(slice_file.filename.clone(), slice_file);
        }
    }

    // Update the diagnostic reporter with the slice files that contain the file level ignoreWarnings attribute.
    data.diagnostic_reporter.file_level_ignored_warnings = file_ignored_warnings_map(&data.files);

    patch_ast(data)
}

fn try_parse_string(file: &str, raw_text: &str, is_source: bool, data: &mut CompilationData) -> Result<SliceFile, ()> {
    let ast = &mut data.ast;
    let diagnostic_reporter = &mut data.diagnostic_reporter;

    // Run the raw text through the preprocessor.
    let mut defined_symbols = HashSet::new();
    let mut preprocessor = crate::parsers::Preprocessor::new(file, &mut defined_symbols, diagnostic_reporter);
    let preprocessed_text = preprocessor.parse_slice_file(raw_text)?;

    // Run the preprocessed text through the parser.
    let mut parser = crate::parsers::Parser::new(file, ast, diagnostic_reporter);
    let (file_encoding, file_attributes, modules) = parser.parse_slice_file(preprocessed_text)?;

    // Add the top-level-modules into the AST, but keep `WeakPtr`s to them.
    let top_level_modules = modules
        .into_iter()
        .map(|module| ast.add_named_element(module))
        .collect::<Vec<_>>();

    Ok(SliceFile::new(
        file.to_owned(),
        raw_text.to_owned(),
        top_level_modules,
        file_attributes,
        file_encoding,
        is_source,
    ))
}

fn patch_ast(mut compilation_data: CompilationData) -> CompilationResult {
    // TODO integrate this better with CompilationData in the future.
    if !compilation_data.has_errors() {
        unsafe {
            compilation_data = crate::ast::patch_ast(compilation_data)?;
        }
    }

    compilation_data.into()
}

// Returns a HashMap where the keys are the relative paths of the .slice files that have the file level
// `ignoreWarnings` attribute and the values are the arguments of the attribute.
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
