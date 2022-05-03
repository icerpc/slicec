// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::error::ErrorReporter;
use slice::parse_from_string;

/// This function is used to parse a Slice file and return the AST.
pub fn parse_for_ast(slice: &str) -> Ast {
    match parse_from_string(slice) {
        Ok((ast, error_reporter)) => {
            assert!(!error_reporter.has_errors(true));
            ast
        }
        Err(e) => panic!("{:?}", e),
    }
}

/// This function is used to parse a Slice file and return the ErrorReporter.
pub fn parse_for_errors(slice: &str) -> ErrorReporter {
    match parse_from_string(slice) {
        Ok((_, error_reporter)) => error_reporter,
        Err(e) => panic!("{:?}", e),
    }
}
