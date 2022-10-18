// Copyright (c) ZeroC, Inc. All rights reserved.

//! This module defines all the tokens and errors that the Slice [Lexer](super::lexer::Lexer) can return.

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

    /// A string of consecutive numeric characters.
    IntegerLiteral(&'input str), // "[0-9]+"

    /// A string literal consists of any characters contained within a pair of unescaped double-quotes.
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
    TraitKeyword,     // "trait"
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
    StringKeyword,    // "String"
    AnyClassKeyword,  // "AnyClass"

    // Other keywords
    TagKeyword,        // "tag"
    StreamKeyword,     // "stream"
    CompactKeyword,    // "compact"
    IdempotentKeyword, // "idempotent"
    UncheckedKeyword,  // "unchecked"
    EncodingKeyword,   // "encoding"

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
    Semicolon,    // ";"
    Equals,       // "="
    QuestionMark, // "?"
    Arrow,        // "->"
    Plus,         // "+"
    Minus,        // "-"
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

    /// Returned when a string is missing its closing quotation mark.
    /// Ex: `"this is a bad string`, there's no closing '"' before EOF.
    UnterminatedStringLiteral,

    /// Returned when a block comment is missing its closing "*/".
    /// Ex: `/* this is a bad comment`, there's no closing "*/" before EOF.
    UnterminatedBlockComment,
}
