// Copyright (c) ZeroC, Inc.

//! This module defines all the tokens and errors that the comment [Lexer](super::lexer::Lexer) can return.

use crate::slice_file::Location;
use std::fmt;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error<'a> = (Location, ErrorKind<'a>, Location);

/// This enum specifies all the kinds of tokens that the comment [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// An identifier for a Slice definition. This rule is more flexible than Slice allows but this keeps the lexer
    /// simpler. This is fine because we validate that the identifier corresponds to a real identifier later.
    Identifier(&'input str), // "[_a-zA-Z0-9]+"

    /// Raw text that isn't parsed any further. These tokens make up messages and descriptions and are only lexed on
    /// lines that continue another section, or after a ':' character on lines that start a new section.
    Text(&'input str),

    Newline, // "\n"

    // Tag keywords
    ParamKeyword,   // "@param"
    ReturnsKeyword, // "@returns"
    ThrowsKeyword,  // "@throws"
    SeeKeyword,     // "@see"
    LinkKeyword,    // "@link"

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

    /// Returned when a '@' isn't followed by a tag identifier.
    /// Ex: `@ link`, there's a space between the '@' and the identifier.
    MissingTag,

    /// Returned when an inline tag is missing its closing brace '}'. Note that inline tags cannot span multiple lines.
    /// Ex: `{@link MyLink`, there's no closing '}' on this line.
    UnterminatedInlineTag,

    /// Returned when a tag is used in the incorrect context.
    /// Some tags can only be used inline, and others can only be used to start a new section.
    /// Ex: `{@param MyParam}`, param tags can't be used inline, and must be at the start of a new section.
    IncorrectContextForTag { tag: &'input str, is_inline: bool },
}

impl fmt::Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::UnknownSymbol { symbol } => write!(f, "unknown symbol '{symbol}'"),
            Self::UnknownTag { tag } => write!(f, "unknown doc comment tag '{tag}'"),
            Self::MissingTag => f.write_str("missing doc comment tag"),
            Self::UnterminatedInlineTag => f.write_str("missing a closing '}' on an inline doc comment tag"),
            Self::IncorrectContextForTag { tag, is_inline } => f.write_fmt(format_args!(
                "doc comment tag '{tag}' cannot be used {}",
                if *is_inline { "inline" } else { "to start a block" },
            )),
        }
    }
}
