// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::common::{ParserResult, SourceBlock};
use super::lexer::Lexer;
use crate::ast::Ast;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::utils::ptr_util::OwnedPtr;

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty $(,)?) => {
        #[allow(clippy::result_unit_err)]
        pub fn $function_name<'input, T: Iterator<Item = SourceBlock<'input>>>(
            &'a mut self,
            input: impl Into<Lexer<'input, T>>,
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

pub struct Parser<'a> {
    pub file_name: &'a str,
    pub(super) ast: &'a mut Ast,
    pub(super) diagnostic_reporter: &'a mut DiagnosticReporter,
    pub(super) current_scope: Scope,
    pub(super) file_encoding: Encoding,
    pub(super) last_enumerator_value: Option<i64>,
}

impl<'a> Parser<'a> {
    implement_parse_function!(
        parse_slice_file,
        SliceFileParser,
        (Option<FileEncoding>, Vec<Attribute>, Vec<OwnedPtr<Module>>),
    );

    pub fn new(
        file_name: &'a str,
        ast: &'a mut Ast,
        diagnostic_reporter: &'a mut DiagnosticReporter,
    ) -> Self {
        Parser {
            file_name,
            ast,
            diagnostic_reporter,
            file_encoding: Encoding::default(),
            current_scope: Scope::default(),
            last_enumerator_value: None,
        }
    }
}
