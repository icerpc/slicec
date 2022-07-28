// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::errors::ErrorReporter;
use slice::parse_from_string;

/// This function is used to parse a Slice file and return the AST.
pub fn parse_for_ast(slice: impl Into<String>) -> Ast {
    match parse_from_string(&slice.into()) {
        Ok(data) => data.ast,
        Err(e) => panic!("{:?}", e.error_reporter),
    }
}

/// This function is used to parse a Slice file and return the ErrorReporter.
pub fn parse_for_errors(slice: impl Into<String>) -> ErrorReporter {
    match parse_from_string(&slice.into()) {
        Ok(data) => data.error_reporter,
        Err(data) => data.error_reporter,
    }
}

/// This function returns the kind of an element, but pluralized.
pub fn pluralize_kind(s: &str) -> String {
    match s {
        "class" => "classes".to_owned(),
        "type alias" => "type aliases".to_owned(),
        "dictionary" => "dictionaries".to_owned(),
        kind => kind.to_owned() + "s",
    }
}
