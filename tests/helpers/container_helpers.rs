// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::parse_from_string;

pub fn parse(slice: &str) -> Ast {
    let (ast, error_reporter) = parse_from_string(slice).ok().unwrap();
    assert!(!error_reporter.has_errors(true));
    ast
}
