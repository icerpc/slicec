// Copyright (c) ZeroC, Inc.

use super::super::common::{has_errors, ParserResult};
use super::construct_lint_from;
use super::grammar::lalrpop;
use super::lexer::Lexer;
use crate::diagnostics::Diagnostic;
use crate::grammar::DocComment;
use crate::slice_file::Span;

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty $(,)?) => {
        #[allow(clippy::result_unit_err)]
        pub fn $function_name(mut self, input: Vec<(&str, Span)>) -> ParserResult<$return_type> {
            match lalrpop::$underlying_parser::new().parse(&mut self, Lexer::new(input)) {
                Err(parse_error) => {
                    let lint = construct_lint_from(parse_error, self.file_name).set_scope(self.identifier);
                    self.diagnostics.push(lint);
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

pub struct CommentParser<'a> {
    pub file_name: &'a str,
    pub(super) identifier: &'a String,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> CommentParser<'a> {
    implement_parse_function!(parse_doc_comment, DocCommentParser, DocComment);

    pub fn new(file_name: &'a str, identifier: &'a String, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
        CommentParser {
            file_name,
            identifier,
            diagnostics,
        }
    }
}
