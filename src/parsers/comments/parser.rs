// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::common::ParserResult;
use super::lexer::Lexer;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::DocComment;
use crate::slice_file::Span;

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty $(,)?) => {
        #[allow(clippy::result_unit_err)]
        pub fn $function_name<'input>(&'a mut self, input: Vec<(&'input str, Span)>) -> ParserResult<$return_type> {
            super::grammar::lalrpop::$underlying_parser::new()
                .parse(self, Lexer::new(input))
                .map_err(|parse_error| {
                    super::construct_warning_from(parse_error, self.file_name)
                        .set_scope(self.identifier)
                        .report(self.reporter);
                })
        }
    };
}

pub struct CommentParser<'a> {
    pub file_name: &'a str,
    pub(super) identifier: &'a String,
    pub(super) reporter: &'a mut DiagnosticReporter,
}

impl<'a> CommentParser<'a> {
    implement_parse_function!(parse_doc_comment, DocCommentParser, DocComment);

    pub fn new(file_name: &'a str, identifier: &'a String, reporter: &'a mut DiagnosticReporter) -> Self {
        CommentParser {
            file_name,
            identifier,
            reporter,
        }
    }
}
