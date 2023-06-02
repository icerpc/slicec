// Copyright (c) ZeroC, Inc.

use super::super::common::{has_errors, ParserResult, SourceBlock};
use super::construct_error_from;
use super::grammar::lalrpop;
use super::lexer::Lexer;
use crate::ast::Ast;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};

/// Helper macro for generating parsing functions.
macro_rules! implement_parse_function {
    ($function_name:ident, $underlying_parser:ident, $return_type:ty $(,)?) => {
        #[allow(clippy::result_unit_err)]
        pub fn $function_name<'input, T>(mut self, input: impl Into<Lexer<'input, T>>) -> ParserResult<$return_type>
        where
            T: Iterator<Item = SourceBlock<'input>>,
        {
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

pub struct Parser<'a> {
    pub file_name: &'a str,
    pub(super) ast: &'a mut Ast,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
    pub(super) current_scope: Scope,
    pub(super) file_encoding: Encoding,
    pub(super) last_enumerator_value: Option<i128>,
}

impl<'a> Parser<'a> {
    implement_parse_function!(
        parse_slice_file,
        SliceFileParser,
        (
            Option<FileEncoding>,
            Vec<WeakPtr<Attribute>>,
            Option<OwnedPtr<Module>>,
            Vec<Definition>,
        ),
    );

    pub fn new(file_name: &'a str, ast: &'a mut Ast, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
        let current_scope = Scope {
            parser_scope: String::new(),
            module: WeakPtr::create_uninitialized(), // Patched when we see a module.
        };
        Parser {
            file_name,
            ast,
            diagnostics,
            file_encoding: Encoding::default(),
            current_scope,
            last_enumerator_value: None,
        }
    }
}
