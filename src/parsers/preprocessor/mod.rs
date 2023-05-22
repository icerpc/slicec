// Copyright (c) ZeroC, Inc.

pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod tokens;

use self::tokens::TokenKind;
use crate::diagnostics::{Diagnostic, Error};
use crate::slice_file::{Location, Span};

type ParseError<'a> = lalrpop_util::ParseError<Location, TokenKind<'a>, tokens::Error>;

// TODO add more specific error messages for common cases.

/// Converts an [error](tokens::Error) that was emitted from the parser/lexer into an [error](Error) that
/// can be handled by the [`DiagnosticReporter`](crate::diagnostics::DiagnosticReporter).
fn construct_error_from(parse_error: ParseError, file_name: &str) -> Diagnostic {
    match parse_error {
        // A custom error we emitted; See `ErrorKind`.
        ParseError::User {
            error: (start, parse_error_kind, end),
        } => {
            let converted = Error::Syntax {
                message: parse_error_kind.to_string(),
            };
            Diagnostic::new(converted).set_span(&Span::new(start, end, file_name))
        }

        // The parser encountered a token that didn't fit any grammar rule.
        ParseError::UnrecognizedToken {
            token: (start, token_kind, end),
            expected,
        } => {
            let message = format!(
                "expected one of {}, but found '{token_kind:?}'",
                clean_message(&expected)
            );
            Diagnostic::new(Error::Syntax { message }).set_span(&Span::new(start, end, file_name))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEof { location, expected } => {
            let message = format!("expected one of {}, but found 'EOF'", clean_message(&expected));
            Diagnostic::new(Error::Syntax { message }).set_span(&Span::new(location, location, file_name))
        }

        // Only the built-in lexer emits 'InvalidToken' errors. We use our own lexer so this is impossible.
        ParseError::InvalidToken { .. } => panic!("impossible 'InvalidToken' encountered in preprocessor"),

        // Only rules that explicitly match 'EOF' or only match a finite number of tokens can emit this error.
        // None of our rules do, so this is impossible (there's no limit to the length of a slice file's contents).
        ParseError::ExtraToken { .. } => panic!("impossible 'ExtraToken' encountered in preprocessor"),
    }
}

fn clean_message(expected: &[String]) -> String {
    match expected {
        [first] => first.to_owned(),
        [first, second] => format!("{first} or {second}"),
        many => {
            let (last, others) = many.split_last().unwrap();
            format!("{}, or {last}", others.join(", "))
        }
    }
}
