// Copyright (c) ZeroC, Inc.

//! This module defines all the tokens and errors that the Slice [Lexer](super::lexer::Lexer) can return.

use std::fmt;

use crate::diagnostics;
use crate::slice_file::Location;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error = (Location, ErrorKind, Location);

/// This enum specifies all the kinds of tokens that the Slice [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// An identifier for a Slice definition. Valid identifiers contain only underscores and alphanumeric characters,
    /// and the first character must be non-numeric.
    ///
    /// While identifiers can be escaped with a leading '\', this is not counted as part of the identifier.
    Identifier(&'input str), // "[_a-zA-Z][_a-zA-Z0-9]*"

    /// A string of alphanumeric characters that starts with a number.
    /// We allow alphanumeric characters to support hex literals.
    IntegerLiteral(&'input str), // "[0-9][a-zA-Z0-9]*"

    /// A string literal consists of any non-newline characters contained within a pair of unescaped double-quotes.
    /// Note that the value doesn't contain the enclosing quotation marks, only the characters in between them.
    StringLiteral(&'input str),

    /// Documentation comments are preceded by 3 forward slashes ("///") and continue until end of line.
    /// Note that the value doesn't contain the slashes or the newline, only the characters in between them.
    DocComment(&'input str),

    // Definition keywords
    ModuleKeyword,    // "module"
    StructKeyword,    // "struct"
    ExceptionKeyword, // "exception"
    ClassKeyword,     // "class"
    InterfaceKeyword, // "interface"
    EnumKeyword,      // "enum"
    CustomKeyword,    // "custom"
    TypeAliasKeyword, // "typealias"

    // Collection keywords
    SequenceKeyword,   // "sequence"
    DictionaryKeyword, // "dictionary"

    // Primitive type keywords
    BoolKeyword,           // "bool"
    Int8Keyword,           // "int8"
    UInt8Keyword,          // "uint8"
    Int16Keyword,          // "int16"
    UInt16Keyword,         // "uint16"
    Int32Keyword,          // "int32"
    UInt32Keyword,         // "uint32"
    VarInt32Keyword,       // "varint32"
    VarUInt32Keyword,      // "varuint32"
    Int64Keyword,          // "int64"
    UInt64Keyword,         // "uint64"
    VarInt62Keyword,       // "varint62"
    VarUInt62Keyword,      // "varuint62"
    Float32Keyword,        // "float32"
    Float64Keyword,        // "float64"
    StringKeyword,         // "string"
    ServiceAddressKeyword, // "ServiceAddress"
    AnyClassKeyword,       // "AnyClass"

    // Other keywords
    AnyExceptionKeyword, // "AnyException"
    CompactKeyword,      // "compact"
    EncodingKeyword,     // "encoding"
    IdempotentKeyword,   // "idempotent"
    StreamKeyword,       // "stream"
    TagKeyword,          // "tag"
    ThrowsKeyword,       // "throws"
    UncheckedKeyword,    // "unchecked"

    // Brackets
    LeftParenthesis,    // "("
    RightParenthesis,   // ")"
    LeftBracket,        // "["
    RightBracket,       // "]"
    DoubleLeftBracket,  // "[["
    DoubleRightBracket, // "]]"
    LeftBrace,          // "{"
    RightBrace,         // "}"
    LeftChevron,        // "<"
    RightChevron,       // ">"

    // Symbols
    Comma,        // ","
    Colon,        // ":"
    DoubleColon,  // "::"
    Equals,       // "="
    QuestionMark, // "?"
    Arrow,        // "->"
    Minus,        // "-"
}

impl std::fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match &self {
            TokenKind::Identifier(input) => input,     // "[_a-zA-Z][_a-zA-Z0-9]*"
            TokenKind::IntegerLiteral(input) => input, // "[0-9][a-zA-Z0-9]*"
            TokenKind::StringLiteral(input) => input,
            TokenKind::DocComment(input) => input,

            // Definition keywords
            TokenKind::ModuleKeyword => "module",
            TokenKind::StructKeyword => "struct",
            TokenKind::ExceptionKeyword => "exception",
            TokenKind::ClassKeyword => "class",
            TokenKind::InterfaceKeyword => "interface",
            TokenKind::EnumKeyword => "enum",
            TokenKind::CustomKeyword => "custom",
            TokenKind::TypeAliasKeyword => "typealias",

            // Collection keywords
            TokenKind::SequenceKeyword => "sequence",
            TokenKind::DictionaryKeyword => "dictionary",

            // Primitive type keywords
            TokenKind::BoolKeyword => "bool",
            TokenKind::Int8Keyword => "int8",
            TokenKind::UInt8Keyword => "uint8",
            TokenKind::Int16Keyword => "int16",
            TokenKind::UInt16Keyword => "uint16",
            TokenKind::Int32Keyword => "int32",
            TokenKind::UInt32Keyword => "uint32",
            TokenKind::VarInt32Keyword => "varint32",
            TokenKind::VarUInt32Keyword => "varuint32",
            TokenKind::Int64Keyword => "int64",
            TokenKind::UInt64Keyword => "uint64",
            TokenKind::VarInt62Keyword => "varint62",
            TokenKind::VarUInt62Keyword => "varuint62",
            TokenKind::Float32Keyword => "float32",
            TokenKind::Float64Keyword => "float64",
            TokenKind::StringKeyword => "string",
            TokenKind::ServiceAddressKeyword => "ServiceAddress",
            TokenKind::AnyClassKeyword => "AnyClass",

            // Other keywords
            TokenKind::AnyExceptionKeyword => "AnyException",
            TokenKind::CompactKeyword => "compact",
            TokenKind::EncodingKeyword => "encoding",
            TokenKind::IdempotentKeyword => "idempotent",
            TokenKind::StreamKeyword => "stream",
            TokenKind::TagKeyword => "tag",
            TokenKind::ThrowsKeyword => "throws",
            TokenKind::UncheckedKeyword => "unchecked",

            // Brackets
            TokenKind::LeftParenthesis => "(",
            TokenKind::RightParenthesis => ")",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::DoubleLeftBracket => "[[",
            TokenKind::DoubleRightBracket => "]]",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftChevron => "<",
            TokenKind::RightChevron => ">",

            // Symbols
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::DoubleColon => "::",
            TokenKind::Equals => "=",
            TokenKind::QuestionMark => "?",
            TokenKind::Arrow => "->",
            TokenKind::Minus => "-",
        })
    }
}

/// This enum specifies all the kinds of errors that the Slice [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// Returned when an unknown symbol is encountered.
    /// If the unknown symbol is similar to a valid symbol, or can be used validly in a different context, the parser
    /// will suggest the valid alternative.
    /// Ex: `$` isn't a valid symbol, and isn't similar to any valid symbols. No suggestion will be supplied.
    /// Ex: `-` isn't a valid symbol, but "->" is a valid symbol. So the parser will suggest "->` to the user.
    UnknownSymbol { symbol: String, suggestion: Option<String> },

    /// Returned when a string is missing its closing quotation mark. Note that strings cannot span multiple lines.
    /// Ex: `"this is a bad string`, there's no closing '"' before EOL.
    UnterminatedStringLiteral,

    /// Returned when a block comment is missing its closing "*/".
    /// Ex: `/* this is a bad comment`, there's no closing "*/" before EOF.
    UnterminatedBlockComment,
}

impl From<ErrorKind> for diagnostics::Error {
    fn from(kind: ErrorKind) -> diagnostics::Error {
        let kind = match kind {
            ErrorKind::UnknownSymbol { symbol, suggestion } => diagnostics::ErrorKind::Syntax {
                message: match suggestion {
                    Some(s) => format!("unknown symbol '{symbol}', try using '{s}' instead"),
                    None => format!("unknown symbol '{symbol}'"),
                },
            },
            ErrorKind::UnterminatedStringLiteral => diagnostics::ErrorKind::Syntax {
                message: "unterminated string literal".to_owned(),
            },
            ErrorKind::UnterminatedBlockComment => diagnostics::ErrorKind::Syntax {
                message: "unterminated block comment".to_owned(),
            },
        };
        diagnostics::Error::new(kind)
    }
}
