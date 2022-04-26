// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::error::ErrorReporter;
use slice::parse_from_string;

/// This function is used to parse a Slice file and return the AST.
pub fn parse_for_ast(slice: &str) -> Ast {
    let (ast, error_reporter) = parse_from_string(slice).ok().unwrap();
    assert!(!error_reporter.has_errors(true));
    ast
}

/// This function is used to parse a Slice file and return the ErrorReporter.
pub fn parse_for_errors(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();
    error_reporter
}
