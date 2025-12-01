// Copyright (c) ZeroC, Inc.

pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod tokens;

use self::tokens::{Error, TokenKind};
use crate::diagnostics::{Diagnostic, Lint};
use crate::slice_file::{Location, Span};

type ParseError<'a> = lalrpop_util::ParseError<Location, TokenKind<'a>, Error<'a>>;

// TODO add more specific error messages for common cases.

/// Converts an [error](Error) that was emitted from the parser/lexer into a [lint](Lint)
/// that can be stored in a [`Diagnostics`](crate::diagnostics::Diagnostics) struct.
fn construct_lint_from(parse_error: ParseError, file_name: &str) -> Diagnostic {
    match parse_error {
        // A custom error we emitted; See `ErrorKind`.
        ParseError::User {
            error: (start, parse_error_kind, end),
        } => {
            let converted = Lint::MalformedDocComment {
                message: parse_error_kind.to_string(),
            };
            Diagnostic::new(converted).set_span(&Span::new(start, end, file_name))
        }

        // The parser encountered a token that didn't fit any grammar rule.
        ParseError::UnrecognizedToken {
            token: (start, token_kind, end),
            expected,
        } => {
            // TODO: should use Display like in Slice parser.
            let message = generate_message(&expected, token_kind);
            Diagnostic::new(Lint::MalformedDocComment { message }).set_span(&Span::new(start, end, file_name))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEof { location, expected } => {
            let message = generate_message(&expected, "EOF");
            Diagnostic::new(Lint::MalformedDocComment { message }).set_span(&Span::new(location, location, file_name))
        }

        _ => unreachable!("impossible error encountered in comment parser: {parse_error:?}"),
    }
}

// TODO: we should convert the LALRpop keywords to human words like we do for the Slice parser.
// TODO: this is identical to the bottom of parsers/slice/mod.rs, we should roll them into a helper function.
fn generate_message(expected: &[String], found: impl std::fmt::Debug) -> String {
    let expected_message = match expected {
        [] => "expected no tokens".to_owned(),
        [first] => format!("expected {first}"),
        [first, second] => format!("expected one of {first} or {second}"),
        many => {
            let (last, others) = many.split_last().unwrap();
            format!("expected one of {}, or {last}", others.join(", "))
        }
    };
    format!("{expected_message}, but found '{found:?}'")
}
