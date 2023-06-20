// Copyright (c) ZeroC, Inc.

use super::super::common::{has_errors, ParserResult, SourceBlock};
use super::construct_error_from;
use super::grammar::lalrpop;
use super::lexer::Lexer;
use crate::diagnostics::Diagnostic;
use std::collections::HashSet;

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty $(,)?) => {
        #[allow(clippy::result_unit_err)]
        pub fn $function_name<'input>(mut self, input: impl Into<Lexer<'input>>) -> ParserResult<$return_type> {
            match lalrpop::$underlying_parser::new().parse(&mut self, input.into()) {
                Err(parse_error) => {
                    let error = construct_error_from(parse_error, self.file_name);
                    self.diagnostics.push(error);
                    Err(())
                }
                Ok(parse_value) => match has_errors(self.diagnostics) {
                    false => Ok(parse_value),
                    true => Err(()),
                },
            }
        }
    };
}

pub struct Preprocessor<'a> {
    pub file_name: &'a str,
    pub(super) defined_symbols: &'a mut HashSet<String>,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> Preprocessor<'a> {
    implement_parse_function!(
        parse_slice_file,
        SliceFileParser,
        impl Iterator<Item = SourceBlock<'input>>,
    );

    pub fn new(
        file_name: &'a str,
        defined_symbols: &'a mut HashSet<String>,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Preprocessor {
            file_name,
            defined_symbols,
            diagnostics,
        }
    }
}
