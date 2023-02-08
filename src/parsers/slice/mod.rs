// Copyright (c) ZeroC, Inc.

pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod tokens;

use crate::diagnostics;
use crate::slice_file::{Location, Span};

type ParseError<'a> = lalrpop_util::ParseError<Location, tokens::TokenKind<'a>, tokens::Error>;

// TODO add more specific error messages for common cases.

/// Converts an [error](tokens::Error) that was emitted from the parser/lexer into an [error](diagnostics::Error)
/// that can be handled by the [`DiagnosticReporter`](diagnostics::DiagnosticReporter).
fn construct_error_from(parse_error: ParseError, file_name: &str) -> diagnostics::Error {
    match parse_error {
        // A custom error we emitted; See `tokens::ErrorKind`.
        ParseError::User {
            error: (start, parse_error_kind, end),
        } => diagnostics::Error::from(parse_error_kind).set_span(&Span::new(start, end, file_name)),

        // The parser encountered a token that didn't fit any grammar rule.
        ParseError::UnrecognizedToken {
            token: (start, token_kind, end),
            expected,
        } => {
            let message = format!("expected one of {}, but found '{token_kind}'", clean_message(&expected));
            diagnostics::Error::new(diagnostics::ErrorKind::Syntax { message })
                .set_span(&Span::new(start, end, file_name))
        }

        // The parser hit EOF in the middle of a grammar rule.
        ParseError::UnrecognizedEOF { location, expected } => {
            let message = format!("expected one of {}, but found 'EOF'", clean_message(&expected));
            diagnostics::Error::new(diagnostics::ErrorKind::Syntax { message })
                .set_span(&Span::new(location, location, file_name))
        }

        // Only the built-in lexer emits 'InvalidToken' errors. We use our own lexer so this is impossible.
        ParseError::InvalidToken { .. } => panic!("impossible 'InvalidToken' encountered in preprocessor"),

        // Only rules that explicitly match 'EOF' or only match a finite number of tokens can emit this error.
        // None of our rules do, so this is impossible (there's no limit to the length of a slice file's contents).
        ParseError::ExtraToken { .. } => panic!("impossible 'ExtraToken' encountered in preprocessor"),
    }
}

fn clean_message(expected: &[String]) -> String {
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
            "service_address_keyword" => tokens::TokenKind::ServiceAddressKeyword.to_string(),
            "any_class_keyword" => tokens::TokenKind::AnyClassKeyword.to_string(),

            // Other keywords
            "any_exception_keyword" => tokens::TokenKind::AnyExceptionKeyword.to_string(),
            "compact_keyword" => tokens::TokenKind::CompactKeyword.to_string(),
            "encoding_keyword" => tokens::TokenKind::EncodingKeyword.to_string(),
            "idempotent_keyword" => tokens::TokenKind::IdempotentKeyword.to_string(),
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
            "\";\"" => tokens::TokenKind::Semicolon.to_string(),
            "\"=\"" => tokens::TokenKind::Equals.to_string(),
            "\"?\"" => tokens::TokenKind::QuestionMark.to_string(),
            "\"->\"" => tokens::TokenKind::Arrow.to_string(),
            "\"-\"" => tokens::TokenKind::Minus.to_string(),
            _ => s.to_owned(),
        })
        .map(|s| format!("'{s}'"))
        .collect::<Vec<String>>();

    match &keyword[..] {
        [first] => first.to_owned(),
        [first, second] => format!("{first} or {second}"),
        many => {
            let (last, others) = many.split_last().unwrap();
            format!("{}, or {last}", others.join(", "))
        }
    }
}
