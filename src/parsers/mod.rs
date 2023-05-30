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
use crate::diagnostics::{Diagnostic, Error};
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

    // Parse the preprocessed text.
    let parser = Parser::new(&file.relative_path, ast, diagnostics);
    let Ok((encoding, attributes, module, definitions)) = parser.parse_slice_file(preprocessed_text) else { return; };

    // Issue a syntax error if the user had definitions but forgot to declare a module.
    if !definitions.is_empty() && module.is_none() {
        let diagnostic = Diagnostic::new(Error::Syntax {
            // TODO improve this message, see: #348
            message: "expected one of '[', '[[', 'doc comment', 'encoding', or 'module', but found 'custom'".to_owned(),
        });
        diagnostics.push(diagnostic);
    }

    // Store the parsed data in the `SliceFile` it was parsed from.
    file.encoding = encoding;
    file.module = module.map(|m| ast.add_named_element(m)); // TODO maybe we don't need to add modules as named anymore?
    file.attributes = attributes;
    file.contents = definitions;
}
