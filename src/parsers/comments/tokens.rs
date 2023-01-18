// Copyright (c) ZeroC, Inc. All rights reserved.

//! This module defines all the tokens and errors that the comment [Lexer](super::lexer::Lexer) can return.

use crate::slice_file::Location;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error<'a> = (Location, ErrorKind<'a>, Location);

/// This enum specifies all the kinds of tokens that the comment [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// An identifier for a Slice definition. This rule is more flexible than the Slice lexer uses to keep this lexer
    /// simpler. This is fine because we validate that the identifier corresponds to a real identifier later.
    Identifier(&'input str), // "[_a-zA-Z0-9]+"

    /// Raw text that isn't parsed any further. These tokens make up messages and descriptions.
    /// These tokens are only lexed on lines that continue another section,
    /// or after a ':' character on lines that start a new section.
    Text(&'input str),

    Newline, // "\n"

    // Tag keywords
    ParamKeyword,   // "@\s*param"
    ReturnsKeyword, // "@\s*returns"
    ThrowsKeyword,  // "@\s*throws"
    SeeKeyword,     // "@\s*see"
    LinkKeyword,    // "@\s*link"

    // Symbols
    LeftBrace,   // "{"
    RightBrace,  // "}"
    Colon,       // ":"
    DoubleColon, // ":"
}

/// This enum specifies all the kinds of errors that the comment [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum ErrorKind<'input> {
    /// Returned when an unknown symbol is encountered in a tag.
    /// Ex: `@param ( foo`, '(' can't validly appear in a tag.
    UnknownSymbol { symbol: char },

    /// Returned when an unknown tag was specified.
    /// Ex: `@foo`, "foo" isn't a valid tag.
    UnknownTag { tag: &'input str },

    /// Returned when a '@' isn't followed by a tag identifier (ignoring whitespace).
    /// Ex: `@`, nothing follows the '@' on this line.
    MissingTag,

    /// Returned when an inline tag is missing its closing brace '}'. Note that inline tags cannot span multiple lines.
    /// Ex: `{@link MyLink`, there's no closing '}' on this line.
    UnterminatedInlineTag,

    /// Returned when a tag is used in the incorrect context.
    /// Some tags can only be used inline, and others can only be used to start a new section.
    /// Ex: `{@param MyParam}`, param tags can't be used inline, and must be at the start of a new section.
    IncorrectContextForTag { tag: &'input str, is_inline: bool },
}
