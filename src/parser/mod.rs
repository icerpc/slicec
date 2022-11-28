// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod comments;

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::DiagnosticReporter;
use crate::slice_file::SliceFile;
use crate::utils::file_util;

use std::collections::HashSet;

// NOTE! it is NOT safe to call any methods on any of the slice entities during parsing.
// Slice entities are NOT considered fully constructed until AFTER parsing is finished (including
// patching). Accessing ANY data, or calling ANY methods before this point may result in panics or
// undefined behavior.

pub fn parse_files(options: &SliceOptions) -> CompilationResult {
    // Create an instance of `CompilationData` for holding all the compiler's state.
    let mut data = CompilationData::create(options);

    // Recursively resolve any Slice files contained in the paths specified by the user.
    let files = file_util::resolve_files_from(options, &mut data.diagnostic_reporter);

    // If any files were unreadable, return without parsing. Otherwise, parse the files normally.
    match data.diagnostic_reporter.has_errors() {
        true => data.into(),
        false => parse_files_impl(files, data, options),
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
        definitions: vec![],
    });

    // Create an instance of `CompilationData` for holding all the compiler's state.
    let data = CompilationData::create(&slice_options);

    // Create a Slice file from each of the strings.
    let mut files = Vec::new();
    for (i, &input) in inputs.iter().enumerate() {
        files.push(SliceFile::new(format!("string-{i}"), input.to_owned(), false))
    }

    parse_files_impl(files, data, &slice_options)
}

fn parse_files_impl(mut files: Vec<SliceFile>, mut data: CompilationData, options: &SliceOptions) -> CompilationResult {
    // Retrieve any preprocessor symbols defined by the compiler itself, or by the user on the command line.
    let symbols = HashSet::from_iter(options.definitions.iter().cloned());

    // Parse the files.
    for file in &mut files {
        parse_slice_file(file, &mut data.ast, &mut data.diagnostic_reporter, symbols.clone());
    }

    // Convert the `Vec<file object>` into a `HashMap<absolute_path, file object>` for easier lookup, and store it.
    data.files = files.into_iter().map(|file| (file.filename.clone(), file)).collect();
    data.into()
}

fn parse_slice_file(
    file: &mut SliceFile,
    ast: &mut Ast,
    diagnostic_reporter: &mut DiagnosticReporter,
    mut symbols: HashSet<String>,
) {
    let mut preprocessor = crate::parsers::Preprocessor::new(&file.filename, &mut symbols, diagnostic_reporter);
    let Ok(preprocessed_text) = preprocessor.parse_slice_file(file.raw_text.as_str()) else { return; };

    // Run the preprocessed text through the parser.
    let mut parser = crate::parsers::Parser::new(&file.filename, ast, diagnostic_reporter);
    let Ok((encoding, attributes, modules)) = parser.parse_slice_file(preprocessed_text) else { return; };

    // Add the top-level-modules into the AST, but keep `WeakPtr`s to them.
    let top_level_modules = modules
        .into_iter()
        .map(|module| ast.add_named_element(module))
        .collect::<Vec<_>>();

    // Store the parsed data in the `SliceFile` it was parsed from.
    file.encoding = encoding;
    file.attributes = attributes;
    file.contents = top_level_modules;
}
