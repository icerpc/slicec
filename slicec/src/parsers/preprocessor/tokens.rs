// Copyright (c) ZeroC, Inc.

//! This module defines all the tokens and errors that the preprocessor [Lexer](super::lexer::Lexer) can return.

use super::super::common::SourceBlock;
use crate::slice_file::Location;
use std::fmt;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error = (Location, ErrorKind, Location);

/// This enum specifies all the kinds of tokens that the preprocessor [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// A block of contiguous Slice source code (as opposed to a preprocessor directive).
    /// A Slice file is comprised of lines of preprocessor directives with blocks of source code between them.
    /// The preprocessor preserves these blocks untouched, and performs no analysis or parsing of them.
    SourceBlock(SourceBlock<'input>),

    /// An identifier for a preprocessor variable, which may either be defined (true) or undefined (false).
    Identifier(&'input str), // "[a-zA-Z][_a-zA-Z0-9]*"

    // Directive keywords
    DefineKeyword,   // "#\s*define"
    UndefineKeyword, // "#\s*undef"
    IfKeyword,       // "#\s*if"
    ElifKeyword,     // "#\s*elif"
    ElseKeyword,     // "#\s*else"
    EndifKeyword,    // "#\s*endif"

    DirectiveEnd,

    // Operators
    Not, // "!"
    And, // "&&"
    Or,  // "||"

    // Brackets
    LeftParenthesis,  // "("
    RightParenthesis, // ")"
}

/// This enum specifies all the kinds of errors that the preprocessor [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// Returned when an unknown symbol is encountered.
    /// If the unknown symbol is similar to a valid operator, the preprocessor will suggest the valid operator.
    /// Ex: `#if (foo + bar)`, '+' isn't a valid operator. No suggestion will be supplied.
    /// Ex: `#if (foo & bar)`, '&' isn't valid, but '&&' is valid. The preprocessor will suggest '&&' to the user.
    UnknownSymbol { symbol: String, suggestion: Option<String> },

    /// Returned when an unknown directive was specified.
    /// Ex: `#foo`, "foo" isn't a valid directive.
    UnknownDirective { keyword: String },

    /// Returned when a '#' isn't followed by a directive identifier (ignoring whitespace).
    /// Ex: `#`, nothing follows after the '#'.
    MissingDirective,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSymbol { symbol, suggestion } => match suggestion {
                Some(s) => write!(f, "unknown symbol '{symbol}', try using '{s}' instead"),
                None => write!(f, "unknown symbol '{symbol}'"),
            },
            Self::UnknownDirective { keyword } => write!(f, "unknown preprocessor directive: '{keyword}'"),
            Self::MissingDirective => f.write_str("missing preprocessor directive"),
        }
    }
}
