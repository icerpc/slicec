// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod tokens;

use self::tokens::{Error, ErrorKind, TokenKind};
use crate::diagnostics::{Warning, WarningKind};
use crate::slice_file::{Location, Span};

type ParseError<'a> = lalrpop_util::ParseError<Location, TokenKind<'a>, Error<'a>>;

// TODO add more specific error messages for common cases.

/// Converts an [error](Error) that was emitted from the parser/lexer into a [warning](Warning) that
/// can be handled by the [`DiagnosticReporter`](DiagnosticReporter).
fn construct_warning_from(parse_error: ParseError, file_name: &str) -> Warning {
    match parse_error {
        // A custom error we emitted; See `ErrorKind`.
        ParseError::User {
            error: (start, parse_error_kind, end),
        } => {
            let warning_kind = match parse_error_kind {
                ErrorKind::UnknownSymbol { symbol } => WarningKind::DocCommentSyntax {
                    message: format!("unknown symbol '{symbol}'"),
                },
                ErrorKind::UnknownTag { tag } => WarningKind::UnknownDocCommentTag { tag: tag.to_owned() },
                ErrorKind::MissingTag => WarningKind::MissingDocCommentTag,
                ErrorKind::UnterminatedInlineTag => WarningKind::UnterminatedInlineTag,
                ErrorKind::IncorrectContextForTag { tag, is_inline } => WarningKind::InvalidDocCommentTagUsage {
                    tag: tag.to_owned(),
                    is_inline,
                },
            };
            Warning::new(warning_kind).set_span(&Span::new(start, end, file_name))
        }

        // The parser encountered a token that didn't fit any grammar rule.
        ParseError::UnrecognizedToken {
            token: (start, token_kind, end),
            expected,
        } => {
            let message = format!(
                "expected one of {}, but found '{token_kind:?}'", // TODO: should use Display like in Slice parser.
                clean_message(&expected)
            );
            Warning::new(WarningKind::DocCommentSyntax { message }).set_span(&Span::new(start, end, file_name))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEOF { location, expected } => {
            let message = format!("expected one of {}, but found 'EOF'", clean_message(&expected));
            Warning::new(WarningKind::DocCommentSyntax { message }).set_span(&Span::new(location, location, file_name))
        }

        _ => unreachable!("impossible error encounted in comment parser: {parse_error:?}"),
    }
}

// TODO: we should convert the LALRpop keywords to human words like we do for the Slice parser.
// TODO: this is identical to the bottom of parsers/slice/mod.rs, we should roll them into a helper function.
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
