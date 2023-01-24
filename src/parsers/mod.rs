// Copyright (c) ZeroC, Inc. All rights reserved.

//! TODO write a comment about how parsing works in Slice.

// We only export the preprocessor and parser to keep all the other logic private.
pub use self::preprocessor::parser::Preprocessor;
pub use self::slice::parser::Parser;

mod common;
mod preprocessor;
mod slice;

use crate::ast::Ast;
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::DiagnosticReporter;
use crate::slice_file::SliceFile;
use std::collections::HashSet;

pub fn parse_files(mut data: CompilationData, symbols: &HashSet<String>) -> CompilationResult {
    for file in data.files.values_mut() {
        parse_file(file, &mut data.ast, &mut data.diagnostic_reporter, symbols.clone());
    }
    data.into()
}

fn parse_file(
    file: &mut SliceFile,
    ast: &mut Ast,
    diagnostic_reporter: &mut DiagnosticReporter,
    mut symbols: HashSet<String>,
) {
    // Pre-process the file's raw text.
    let mut preprocessor = Preprocessor::new(&file.relative_path, &mut symbols, diagnostic_reporter);
    let Ok(preprocessed_text) = preprocessor.parse_slice_file(file.raw_text.as_str()) else { return; };

    // If no text remains after pre-processing, the file is empty and we can skip parsing and exit early.
    // To check the length of the preprocessed text without consuming the iterator we convert it to a peekable iterator,
    // then check the peek value.
    let mut peekable_preprocessed_text = preprocessed_text.peekable();
    if peekable_preprocessed_text.peek().is_none() {
        return;
    }

    // Parse the preprocessed text.
    let mut parser = Parser::new(&file.relative_path, ast, diagnostic_reporter);
    let Ok((encoding, attributes, modules)) = parser.parse_slice_file(peekable_preprocessed_text) else { return; };

    // Add the top-level-modules into the AST, but keep `WeakPtr`s to them.
    let top_level_modules = modules
        .into_iter()
        .map(|module| ast.add_named_element(module))
        .collect::<Vec<_>>();

    // Store the parsed data in the `SliceFile` it was parsed from.
    file.encoding = encoding;
    file.attributes = attributes;
    file.contents = top_level_modules;

    // Update the file level ignored warnings map in the diagnostic reporter.
    diagnostic_reporter.add_file_level_ignore_warnings_for(file);
}
