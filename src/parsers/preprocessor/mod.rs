// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod tokens;

use crate::diagnostics;
use crate::slice_file::{Location, Span};

type ParseError<'a> = lalrpop_util::ParseError<Location, tokens::TokenKind<'a>, tokens::Error>;

// TODO add more specific error messages for common cases.

/// Converts an [error](tokens::Error) that was emitted from the parser/lexer into an [error](diagnostics::Error) that
/// can be handled by the [`DiagnosticReporter`](diagnostics::DiagnosticReporter).
fn construct_error_from(parse_error: ParseError, file_name: &str) -> diagnostics::Error {
    match parse_error {
        // A custom error we emitted; See `tokens::ErrorKind`.
        ParseError::User { error: (start, parse_error_kind, end) } => {
            let error_kind = match parse_error_kind {
                tokens::ErrorKind::MissingDirective => {
                    diagnostics::ErrorKind::Syntax("missing preprocessor directive".to_owned())
                }
                tokens::ErrorKind::UnknownDirective { keyword } => {
                    diagnostics::ErrorKind::Syntax(format!("unknown preprocessor directive: '{keyword}'"))
                }
                tokens::ErrorKind::UnknownSymbol { symbol, suggestion } => diagnostics::ErrorKind::Syntax(
                    match suggestion {
                        Some(s) => format!("unknown symbol '{symbol}', try using '{s}' instead"),
                        None => format!("unknown symbol '{symbol}'"),
                    }
                ),
            };
            let span = Span::new(start, end, file_name);
            diagnostics::Error::new(error_kind, Some(&span))
        }

        // The parser encountered a token that didn't fit any grammar rule.
        ParseError::UnrecognizedToken { token: (start, token_kind, end), expected } => {
            let message = format!("expected one of {}, but found '{token_kind:?}'", expected.join(", "));
            let span = Span::new(start, end, file_name);
            diagnostics::Error::new(diagnostics::ErrorKind::Syntax(message), Some(&span))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEOF { location, expected } => {
            let message = format!("expected one of {}, but found 'EOF'", expected.join(", "));
            let span = Span::new(location, location, file_name);
            diagnostics::Error::new(diagnostics::ErrorKind::Syntax(message), Some(&span))
        }

        // Only the built-in lexer emits 'InvalidToken' errors. We use our own lexer so this is impossible.
        ParseError::InvalidToken { .. } => panic!("impossible 'InvalidToken' encountered in preprocessor"),

        // Only rules that explicitly match 'EOF' or only match a finite number of tokens can emit this error.
        // None of our rules do, so this is impossible (there's no limit to the length of a slice file's contents).
        ParseError::ExtraToken { .. } => panic!("impossible 'ExtraToken' encountered in preprocessor"),
    }
}
