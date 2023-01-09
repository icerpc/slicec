// Copyright (c) ZeroC, Inc. All rights reserved.

//! This module defines all the tokens and errors that the preprocessor [Lexer](super::lexer::Lexer) can return.

use super::super::common::SourceBlock;
use crate::diagnostics;
use crate::slice_file::Location;

pub type Token<'a> = (Location, TokenKind<'a>, Location);
pub type Error = (Location, ErrorKind, Location);

/// This enum specifies all the kinds of tokens that the preprocessor [Lexer](super::lexer::Lexer) can return.
#[derive(Clone, Debug)]
pub enum TokenKind<'input> {
    /// An identifier for a preprocessor variable, which may be either defined (true) or undefined (false).
    Identifier(&'input str), // "[_a-zA-Z][_a-zA-Z0-9]*"

    /// A block of contiguous Slice source code (as opposed to a preprocessor directive).
    /// A Slice file is comprised of lines of preprocessor directives with blocks of source code between them.
    /// The preprocessor preserves these blocks untouched, and performs no analysis or parsing of them.
    SourceBlock(SourceBlock<'input>),

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
    /// Returned when a '#' isn't followed by a directive identifier (ignoring whitespace).
    /// Ex: `#`, nothing follows after the '#'.
    MissingDirective,

    /// Returned when an unknown directive was specified.
    /// Ex: `#foo`, 'foo' isn't a valid directive.
    UnknownDirective { keyword: String },

    /// Returned when an unknown symbol is encountered.
    /// If the unknown symbol is similar to a valid operator, the preprocessor will suggest the valid operator.
    /// Ex: `#if (foo + bar)`, '+' isn't a valid operator. No suggestion will be supplied.
    /// Ex: `#if (foo & bar)`, '&' isn't valid, but '&&' is valid. The preprocessor will suggest '&&' to the user.
    UnknownSymbol { symbol: String, suggestion: Option<String> },
}

impl From<ErrorKind> for diagnostics::Error {
    fn from(kind: ErrorKind) -> diagnostics::Error {
        let kind = match kind {
            ErrorKind::MissingDirective => diagnostics::ErrorKind::Syntax {
                message: "missing preprocessor directive".to_owned(),
            },
            ErrorKind::UnknownDirective { keyword } => diagnostics::ErrorKind::Syntax {
                message: format!("unknown preprocessor directive: '{keyword}'"),
            },
            ErrorKind::UnknownSymbol { symbol, suggestion } => diagnostics::ErrorKind::Syntax {
                message: match suggestion {
                    Some(s) => format!("unknown symbol '{symbol}', try using '{s}' instead"),
                    None => format!("unknown symbol '{symbol}'"),
                },
            },
        };
        diagnostics::Error::new(kind)
    }
}
