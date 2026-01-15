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

/// Converts an [error](tokens::Error) that was emitted from the parser/lexer into an [error](Error)
/// that can be stored in a [`Diagnostics`](crate::diagnostics::Diagnostics) struct.
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
            let message = generate_message(&expected, token_kind);
            Diagnostic::new(Error::Syntax { message }).set_span(&Span::new(start, end, file_name))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEof { location, expected } => {
            let message = generate_message(&expected, "EOF");
            Diagnostic::new(Error::Syntax { message }).set_span(&Span::new(location, location, file_name))
        }

        _ => unreachable!("impossible error encountered in Slice parser: '{parse_error:?}'"),
    }
}

// TODO: simplify this or merge the match statements in this function and tokens.rs together.
fn generate_message(expected: &[String], found: impl std::fmt::Display) -> String {
    let keyword = expected
        .iter()
        .map(|s| match s.as_str() {
            "identifier" => "identifier".to_owned(),
            "string_literal" => "string literal".to_owned(),
            "integer_literal" => "integer literal".to_owned(),
            "doc_comment" => "doc comment".to_owned(),

            // Definition keywords
            "module_keyword" => tokens::TokenKind::ModuleKeyword.to_string(),
            "struct_keyword" => tokens::TokenKind::StructKeyword.to_string(),
            "exception_keyword" => tokens::TokenKind::ExceptionKeyword.to_string(),
            "class_keyword" => tokens::TokenKind::ClassKeyword.to_string(),
            "interface_keyword" => tokens::TokenKind::InterfaceKeyword.to_string(),
            "enum_keyword" => tokens::TokenKind::EnumKeyword.to_string(),
            "custom_keyword" => tokens::TokenKind::CustomKeyword.to_string(),
            "type_alias_keyword" => tokens::TokenKind::TypeAliasKeyword.to_string(),
            "result_keyword" => tokens::TokenKind::ResultKeyword.to_string(),

            // Collection keywords
            "sequence_keyword" => tokens::TokenKind::SequenceKeyword.to_string(),
            "dictionary_keyword" => tokens::TokenKind::DictionaryKeyword.to_string(),

            // Primitive type keywords
            "bool_keyword" => tokens::TokenKind::BoolKeyword.to_string(),
            "int8_keyword" => tokens::TokenKind::Int8Keyword.to_string(),
            "uint8_keyword" => tokens::TokenKind::UInt8Keyword.to_string(),
            "int16_keyword" => tokens::TokenKind::Int16Keyword.to_string(),
            "uint16_keyword" => tokens::TokenKind::UInt16Keyword.to_string(),
            "int32_keyword" => tokens::TokenKind::Int32Keyword.to_string(),
            "uint32_keyword" => tokens::TokenKind::UInt32Keyword.to_string(),
            "varint32_keyword" => tokens::TokenKind::VarInt32Keyword.to_string(),
            "varuint32_keyword" => tokens::TokenKind::VarUInt32Keyword.to_string(),
            "int64_keyword" => tokens::TokenKind::Int64Keyword.to_string(),
            "uint64_keyword" => tokens::TokenKind::UInt64Keyword.to_string(),
            "varint62_keyword" => tokens::TokenKind::VarInt62Keyword.to_string(),
            "varuint62_keyword" => tokens::TokenKind::VarUInt62Keyword.to_string(),
            "float32_keyword" => tokens::TokenKind::Float32Keyword.to_string(),
            "float64_keyword" => tokens::TokenKind::Float64Keyword.to_string(),
            "string_keyword" => tokens::TokenKind::StringKeyword.to_string(),
            "any_class_keyword" => tokens::TokenKind::AnyClassKeyword.to_string(),

            // Other keywords
            "compact_keyword" => tokens::TokenKind::CompactKeyword.to_string(),
            "idempotent_keyword" => tokens::TokenKind::IdempotentKeyword.to_string(),
            "mode_keyword" => tokens::TokenKind::ModeKeyword.to_string(),
            "stream_keyword" => tokens::TokenKind::StreamKeyword.to_string(),
            "tag_keyword" => tokens::TokenKind::TagKeyword.to_string(),
            "throws_keyword" => tokens::TokenKind::ThrowsKeyword.to_string(),
            "unchecked_keyword" => tokens::TokenKind::UncheckedKeyword.to_string(),

            // Brackets
            "\"(\"" => tokens::TokenKind::LeftParenthesis.to_string(),
            "\")\"" => tokens::TokenKind::RightParenthesis.to_string(),
            "\"[\"" => tokens::TokenKind::LeftBracket.to_string(),
            "\"]\"" => tokens::TokenKind::RightBracket.to_string(),
            "\"[[\"" => tokens::TokenKind::DoubleLeftBracket.to_string(),
            "\"]]\"" => tokens::TokenKind::DoubleRightBracket.to_string(),
            "\"{\"" => tokens::TokenKind::LeftBrace.to_string(),
            "\"}\"" => tokens::TokenKind::RightBrace.to_string(),
            "\"<\"" => tokens::TokenKind::LeftChevron.to_string(),
            "\">\"" => tokens::TokenKind::RightChevron.to_string(),

            // Symbols
            "\",\"" => tokens::TokenKind::Comma.to_string(),
            "\":\"" => tokens::TokenKind::Colon.to_string(),
            "\"::\"" => tokens::TokenKind::DoubleColon.to_string(),
            "\"=\"" => tokens::TokenKind::Equals.to_string(),
            "\"?\"" => tokens::TokenKind::QuestionMark.to_string(),
            "\"->\"" => tokens::TokenKind::Arrow.to_string(),
            "\"-\"" => tokens::TokenKind::Minus.to_string(),
            _ => s.to_owned(),
        })
        .map(|s| format!("'{s}'"))
        .collect::<Vec<String>>();

    let expected_message = match &keyword[..] {
        [] => "expected no tokens".to_owned(),
        [first] => format!("expected {first}"),
        [first, second] => format!("expected one of {first} or {second}"),
        many => {
            let (last, others) = many.split_last().unwrap();
            format!("expected one of {}, or {last}", others.join(", "))
        }
    };
    format!("{expected_message}, but found '{found}'")
}
