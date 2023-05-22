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

impl fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Identifier(identifier) => write!(f, "identifier"),
            Self::IntegerLiteral(i) => write!(f, "{i}"),
            Self::StringLiteral(s) => write!(f, "{s}"),
            Self::DocComment(_) => f.write_str("doc comment"),

            // Keywords
            Self::ModuleKeyword => f.write_str("module"),
            Self::StructKeyword => f.write_str("struct"),
            Self::ExceptionKeyword => f.write_str("exception"),
            Self::ClassKeyword => f.write_str("class"),
            Self::InterfaceKeyword => f.write_str("interface"),
            Self::EnumKeyword => f.write_str("enum"),
            Self::CustomKeyword => f.write_str("custom"),
            Self::TypeAliasKeyword => f.write_str("typealias"),
            Self::SequenceKeyword => f.write_str("sequence"),
            Self::DictionaryKeyword => f.write_str("dictionary"),
            Self::BoolKeyword => f.write_str("bool"),
            Self::Int8Keyword => f.write_str("int8"),
            Self::UInt8Keyword => f.write_str("uint8"),
            Self::Int16Keyword => f.write_str("int16"),
            Self::UInt16Keyword => f.write_str("uint16"),
            Self::Int32Keyword => f.write_str("int32"),
            Self::UInt32Keyword => f.write_str("uint32"),
            Self::VarInt32Keyword => f.write_str("varint32"),
            Self::VarUInt32Keyword => f.write_str("varuint32"),
            Self::Int64Keyword => f.write_str("int64"),
            Self::UInt64Keyword => f.write_str("uint64"),
            Self::VarInt62Keyword => f.write_str("varint62"),
            Self::VarUInt62Keyword => f.write_str("varuint62"),
            Self::Float32Keyword => f.write_str("float32"),
            Self::Float64Keyword => f.write_str("float64"),
            Self::StringKeyword => f.write_str("string"),
            Self::AnyClassKeyword => f.write_str("AnyClass"),
            Self::AnyExceptionKeyword => f.write_str("AnyException"),
            Self::CompactKeyword => f.write_str("compact"),
            Self::EncodingKeyword => f.write_str("encoding"),
            Self::IdempotentKeyword => f.write_str("idempotent"),
            Self::StreamKeyword => f.write_str("stream"),
            Self::TagKeyword => f.write_str("tag"),
            Self::ThrowsKeyword => f.write_str("throws"),
            Self::UncheckedKeyword => f.write_str("unchecked"),

            // Symbols
            Self::LeftParenthesis => f.write_str("("),
            Self::RightParenthesis => f.write_str(")"),
            Self::LeftBracket => f.write_str("["),
            Self::RightBracket => f.write_str("]"),
            Self::DoubleLeftBracket => f.write_str("[["),
            Self::DoubleRightBracket => f.write_str("]]"),
            Self::LeftBrace => f.write_str("{"),
            Self::RightBrace => f.write_str("}"),
            Self::LeftChevron => f.write_str("<"),
            Self::RightChevron => f.write_str(">"),
            Self::Comma => f.write_str(","),
            Self::Colon => f.write_str(":"),
            Self::DoubleColon => f.write_str("::"),
            Self::Equals => f.write_str("="),
            Self::QuestionMark => f.write_str("?"),
            Self::Arrow => f.write_str("->"),
            Self::Minus => f.write_str("-"),
        }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
