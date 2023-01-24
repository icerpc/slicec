// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::compile_from_strings;
use slice::diagnostics::Diagnostic;

/// This function is used to parse a Slice file and return the AST.
pub fn parse_for_ast(slice: impl Into<String>) -> Ast {
    match compile_from_strings(&[&slice.into()], None) {
        Ok(data) => data.ast,
        Err(e) => panic!("{:?}", e.diagnostic_reporter),
    }
}

/// This function is used to parse a Slice file and return the DiagnosticReporter.
pub fn parse_for_diagnostics(slice: impl Into<String>) -> Vec<Diagnostic> {
    let data = match compile_from_strings(&[&slice.into()], None) {
        Ok(data) => data,
        Err(data) => data,
    };
    data.diagnostic_reporter.into_diagnostics()
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
