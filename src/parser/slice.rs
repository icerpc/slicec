// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::slice_file::SliceFile;
use std::collections::HashSet;
use std::fs;

// TODO: This is a duplicate of 'crate::parsers::common::ParserResult'.
// All this code should be moved into 'parsers/mod.rs' where it can use the real type.
type ParserResult<T> = Result<T, ()>;

pub(super) struct SliceParser<'a> {
    pub diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> SliceParser<'a> {
    pub fn try_parse_file(&mut self, file: &str, is_source: bool, ast: &mut Ast) -> Option<SliceFile> {
        match fs::read_to_string(file) {
            Ok(raw_text) => {
                // The parser emits errors through `DiagnosticReporter` on it's own, so we don't need to handle them.
                self.parse_string(file, &raw_text, is_source, ast).ok()
            }
            Err(err) => {
                self.diagnostic_reporter
                    .report_error(Error::new(ErrorKind::Syntax(err.to_string()), None));
                None
            }
        }
    }

    pub fn try_parse_string(&mut self, identifier: &str, input: &str, ast: &mut Ast) -> Option<SliceFile> {
        // The parser emits errors through `DiagnosticReporter` on it's own, so we don't need to handle them.
        self.parse_string(identifier, input, false, ast).ok()
    }

    fn parse_string(&mut self, file: &str, raw_text: &str, is_source: bool, ast: &mut Ast) -> ParserResult<SliceFile> {
        // Run the raw text through the preprocessor.
        let mut defined_symbols = HashSet::new();
        let mut preprocessor = crate::parsers::Preprocessor::new(file, &mut defined_symbols, self.diagnostic_reporter);
        let preprocessed_text = preprocessor.parse_slice_file(raw_text)?;

        // Run the preprocessed text through the parser.
        let mut parser = crate::parsers::Parser::new(file, ast, self.diagnostic_reporter);
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
}
