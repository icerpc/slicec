// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::utils::ptr_util::OwnedPtr;
use super::lexer::Lexer;
use super::super::common::{ParserResult, SourceBlock};

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty) => {
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
    pub(in super) ast: &'a mut Ast,
    pub(in super) diagnostic_reporter: &'a mut DiagnosticReporter,
    pub(in super) current_scope: Scope,
    pub(in super) file_encoding: Encoding,
    pub(in super) last_enumerator_value: Option<i64>,
}

impl<'a> Parser<'a> {
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

    implement_parse_function!(parse_slice_file, SliceFileParser, (Option<FileEncoding>, Vec<Attribute>, Vec<OwnedPtr<Module>>));
}
