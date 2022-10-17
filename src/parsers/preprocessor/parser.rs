// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::DiagnosticReporter;
use super::lexer::Lexer;
use super::super::common::{ParserResult, SourceBlock};

use std::collections::HashSet;

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty) => {
        pub fn $function_name<'input>(
            &'a mut self,
            input: impl Into<Lexer<'input>>,
        ) -> ParserResult<$return_type> {
            super::grammar::lalrpop::$underlying_parser::new()
                .parse(self, input.into())
                .map_err(|parse_error| {
                    let error = super::construct_error_from(parse_error, self.file_name);
                    self.diagnostic_reporter.report_error(error)
                })
        }
    };
}

pub struct Preprocessor<'a> {
    pub file_name: &'a str,
    pub(in super) definitions: &'a mut HashSet<String>,
    pub(in super) diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> Preprocessor<'a> {
    pub fn new(
        file_name: &'a str,
        definitions: &'a mut HashSet<String>,
        diagnostic_reporter: &'a mut DiagnosticReporter,
    ) -> Self {
        Preprocessor { file_name, definitions, diagnostic_reporter }
    }

    implement_parse_function!(parse_slice_file, SliceFileParser, impl Iterator<Item = SourceBlock<'input>>);
}
