// Copyright (c) ZeroC, Inc.

//! TODO write a comment about how parsing works in Slice.

// We only export the parsers and keep all the other logic private.
pub use self::comments::parser::CommentParser;
pub use self::preprocessor::parser::Preprocessor;
pub use self::slice::parser::Parser;

mod comments;
mod common;
mod preprocessor;
mod slice;

use crate::ast::Ast;
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, DiagnosticReporter};
use crate::slice_file::SliceFile;
use std::collections::HashSet;

pub fn parse_files(state: &mut CompilationState, symbols: &HashSet<String>) {
    for file in state.files.values_mut() {
        parse_file(file, &mut state.ast, &mut state.diagnostic_reporter, symbols.clone());
    }
}

fn parse_file(
    file: &mut SliceFile,
    ast: &mut Ast,
    diagnostic_reporter: &mut DiagnosticReporter,
    symbols: HashSet<String>,
) {
    // Attempt to parse the file.
    let mut diagnostics = Vec::new();
    let _ = try_parse_file(file, ast, &mut diagnostics, symbols);

    // Forward any diagnostics that were emitted during parsing to the diagnostic reporter.
    for diagnostic in diagnostics {
        diagnostic.report(diagnostic_reporter);
    }
}

fn try_parse_file(
    file: &mut SliceFile,
    ast: &mut Ast,
    diagnostics: &mut Vec<Diagnostic>,
    mut symbols: HashSet<String>,
) -> common::ParserResult<()> {
    // Pre-process the file's raw text.
    let preprocessor = Preprocessor::new(&file.relative_path, &mut symbols, diagnostics);
    let preprocessed_text = preprocessor.parse_slice_file(file.raw_text.as_str())?;

    // If no text remains after pre-processing, the file is empty and we can skip parsing and exit early.
    // To check the length of the preprocessed text without consuming the iterator we convert it to a peekable iterator,
    // then check the peek value.
    let mut peekable_preprocessed_text = preprocessed_text.peekable();
    if peekable_preprocessed_text.peek().is_none() {
        return Err(());
    }

    // Parse the preprocessed text.
    let parser = Parser::new(&file.relative_path, ast, diagnostics);
    let (file_encoding, attributes, modules) = parser.parse_slice_file(peekable_preprocessed_text)?;

    // Add the top-level-modules into the AST, but keep `WeakPtr`s to them.
    let top_level_modules = modules
        .into_iter()
        .map(|module| ast.add_named_element(module))
        .collect();

    // Store the parsed data in the `SliceFile` it was parsed from.
    file.encoding = file_encoding;
    file.attributes = attributes;
    file.contents = top_level_modules;
    Ok(())
}
