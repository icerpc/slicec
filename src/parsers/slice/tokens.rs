// Copyright (c) ZeroC, Inc.

//! This module defines all the tokens and errors that the Slice [Lexer](super::lexer::Lexer) can return.

use crate::slice_file::Location;
use std::fmt;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error = (Location, ErrorKind, Location);

/// This enum specifies all the kinds of tokens that the Slice [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// An identifier for a Slice definition. Valid identifiers contain only underscores and alphanumeric characters,
    /// and the first character must be a letter.
    ///
    /// While identifiers can be escaped with a leading '\', this is not counted as part of the identifier.
    Identifier(&'input str), // "[a-zA-Z][_a-zA-Z0-9]*"

    /// A string literal consists of any non-newline characters contained within a pair of unescaped double-quotes.
    /// Note that the value doesn't contain the enclosing quotation marks, only the characters in between them.
    StringLiteral(&'input str),

    /// A string of alphanumeric characters that starts with a number.
    /// We allow alphanumeric characters to support hex literals.
    IntegerLiteral(&'input str), // "[0-9][_a-zA-Z0-9]*"

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
    SequenceKeyword,   // "Sequence"
    DictionaryKeyword, // "Dictionary"

    // Primitive type keywords
    BoolKeyword,      // "bool"
    Int8Keyword,      // "int8"
    UInt8Keyword,     // "uint8"
    Int16Keyword,     // "int16"
    UInt16Keyword,    // "uint16"
    Int32Keyword,     // "int32"
    UInt32Keyword,    // "uint32"
    VarInt32Keyword,  // "varint32"
    VarUInt32Keyword, // "varuint32"
    Int64Keyword,     // "int64"
    UInt64Keyword,    // "uint64"
    VarInt62Keyword,  // "varint62"
    VarUInt62Keyword, // "varuint62"
    Float32Keyword,   // "float32"
    Float64Keyword,   // "float64"
    StringKeyword,    // "string"
    AnyClassKeyword,  // "AnyClass"

    // Other keywords
    CompactKeyword,    // "compact"
    IdempotentKeyword, // "idempotent"
    ModeKeyword,       // "mode"
    StreamKeyword,     // "stream"
    TagKeyword,        // "tag"
    ThrowsKeyword,     // "throws"
    UncheckedKeyword,  // "unchecked"

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

impl fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identifier(input) => input,
            Self::IntegerLiteral(input) => input,
            Self::StringLiteral(input) => input,
            Self::DocComment(input) => input,

            // Keywords
            Self::ModuleKeyword => "module",
            Self::StructKeyword => "struct",
            Self::ExceptionKeyword => "exception",
            Self::ClassKeyword => "class",
            Self::InterfaceKeyword => "interface",
            Self::EnumKeyword => "enum",
            Self::CustomKeyword => "custom",
            Self::TypeAliasKeyword => "typealias",
            Self::SequenceKeyword => "Sequence",
            Self::DictionaryKeyword => "Dictionary",
            Self::BoolKeyword => "bool",
            Self::Int8Keyword => "int8",
            Self::UInt8Keyword => "uint8",
            Self::Int16Keyword => "int16",
            Self::UInt16Keyword => "uint16",
            Self::Int32Keyword => "int32",
            Self::UInt32Keyword => "uint32",
            Self::VarInt32Keyword => "varint32",
            Self::VarUInt32Keyword => "varuint32",
            Self::Int64Keyword => "int64",
            Self::UInt64Keyword => "uint64",
            Self::VarInt62Keyword => "varint62",
            Self::VarUInt62Keyword => "varuint62",
            Self::Float32Keyword => "float32",
            Self::Float64Keyword => "float64",
            Self::StringKeyword => "string",
            Self::AnyClassKeyword => "AnyClass",
            Self::CompactKeyword => "compact",
            Self::IdempotentKeyword => "idempotent",
            Self::ModeKeyword => "mode",
            Self::StreamKeyword => "stream",
            Self::TagKeyword => "tag",
            Self::ThrowsKeyword => "throws",
            Self::UncheckedKeyword => "unchecked",

            // Symbols
            Self::LeftParenthesis => "(",
            Self::RightParenthesis => ")",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::DoubleLeftBracket => "[[",
            Self::DoubleRightBracket => "]]",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::LeftChevron => "<",
            Self::RightChevron => ">",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::DoubleColon => "::",
            Self::Equals => "=",
            Self::QuestionMark => "?",
            Self::Arrow => "->",
            Self::Minus => "-",
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
    /// Ex: `-` isn't a valid symbol, but "->" is a valid symbol. So the parser will suggest "->" to the user.
    UnknownSymbol { symbol: String, suggestion: Option<String> },

    /// Returned when a string is missing its closing quotation mark. Note that strings cannot span multiple lines.
    /// Ex: `"this is a bad string`, there's no closing '"' before EOL.
    UnterminatedStringLiteral,

    /// Returned when a block comment is missing its closing "*/".
    /// Ex: `/* this is a bad comment`, there's no closing "*/" before EOF.
    UnterminatedBlockComment,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSymbol { symbol, suggestion } => match suggestion {
                Some(s) => write!(f, "unknown symbol '{symbol}', try using '{s}' instead"),
                None => write!(f, "unknown symbol '{symbol}'"),
            },
            Self::UnterminatedStringLiteral => f.write_str("unterminated string literal"),
            Self::UnterminatedBlockComment => f.write_str("unterminated block comment"),
        }
    }
}
