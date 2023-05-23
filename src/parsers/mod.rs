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
use crate::diagnostics::Diagnostic;
use crate::slice_file::SliceFile;
use std::collections::HashSet;

pub fn parse_files(state: &mut CompilationState, symbols: &HashSet<String>) {
    for file in state.files.values_mut() {
        // Attempt to parse the file.
        let mut diagnostics = Vec::new();
        parse_file(file, &mut state.ast, &mut diagnostics, symbols.clone());

        // Forward any diagnostics that were emitted during parsing to the diagnostic reporter.
        for diagnostic in diagnostics {
            diagnostic.report(&mut state.diagnostic_reporter);
        }
    }
}

fn parse_file(file: &mut SliceFile, ast: &mut Ast, diagnostics: &mut Vec<Diagnostic>, mut symbols: HashSet<String>) {
    // Pre-process the file's raw text.
    let preprocessor = Preprocessor::new(&file.relative_path, &mut symbols, diagnostics);
    let Ok(preprocessed_text) = preprocessor.parse_slice_file(file.raw_text.as_str()) else { return; };

    // If no text remains after pre-processing, the file is empty and we can skip parsing and exit early.
    // To check the length of the preprocessed text without consuming the iterator we convert it to a peekable iterator,
    // then check the peek value.
    let mut peekable_preprocessed_text = preprocessed_text.peekable();
    if peekable_preprocessed_text.peek().is_none() {
        return;
    }

    // Parse the preprocessed text.
    let parser = Parser::new(&file.relative_path, ast, diagnostics);
    let Ok((file_encoding, attributes, module)) = parser.parse_slice_file(peekable_preprocessed_text) else { return; };

    // Store the parsed data in the `SliceFile` it was parsed from.
    file.encoding = file_encoding;
    file.attributes = attributes;
    file.contents = vec![ast.add_named_element(module)]; // TODOAUSTIN change contents to no longer need a vec!
}
