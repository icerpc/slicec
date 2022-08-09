// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Span;
use std::fmt;

mod error_reporter;
mod logic;
mod warnings;

pub use self::error_reporter::ErrorReporter;
pub use self::logic::LogicKind;
pub use self::warnings::WarningKind;

/// An Error contains information about syntax errors, logic errors, etc., encountered while compiling slice code.
///
/// Each error has a kind, specifying the type of error encountered, such as Syntax, Logic, or IO. Additionally, an
/// Error can have an optional Span which specifies the location in the source code where the error occurred.
#[derive(Debug)]
pub struct Error {
    pub error_kind: ErrorKind,
    pub span: Option<Span>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    /// An error related to the syntax of the slice source code such as missing semicolons or defining classes in a
    /// Slice2 encoded slice file.
    Syntax(String),

    /// An error related to the logic of the slice source code such as using the same tag twice.
    Logic(LogicKind),

    /// A suggestion or warning to aid in preventing a problem. For example warning if a documentation comment
    /// indicates that an operation should return a value, but the operation does not.
    Warning(WarningKind),

    /// Additional information about another kind of error that was encountered. For example, indicating where the
    /// encoding of a Slice1 encoded slice file was defined.
    Note(String),

    /// An error related to the IO of the slice source code such as opening a file that doesn't exist.
    IO(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Syntax(error) => write!(f, "{}", error),
            ErrorKind::Logic(rule_kind) => write!(f, "{}", rule_kind.message()),
            ErrorKind::Warning(warning_kind) => write!(f, "{}", warning_kind.message()),
            ErrorKind::Note(note) => write!(f, "{}", note),
            ErrorKind::IO(error) => write!(f, "{}", error),
        }
    }
}

/// Creates a new note from a string.
///
/// # Examples
/// ```
/// # use slice::errors::ErrorKind;
/// let note = ErrorKind::new_note("This is the content of a note.");
/// ```
impl ErrorKind {
    pub fn new_note(message: impl Into<String>) -> ErrorKind {
        ErrorKind::Note(message.into())
    }
}

#[macro_export]
macro_rules! implement_from_for_error_sub_kind {
    ($type:ty, $enumerator:path) => {
        impl From<$type> for ErrorKind {
            fn from(original: $type) -> ErrorKind {
                $enumerator(original)
            }
        }
    };
}

#[macro_export]
macro_rules! implement_error_functions {
    ($enumerator:ty, $(($kind:path, $code:expr, $message:expr $(, $variant:pat)* )),*) => {
        impl $enumerator {
            pub fn error_code(&self) -> u32 {
                match self {
                    $(
                        implement_error_functions!(@error $kind, $($variant),*) => $code,
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_error_functions!(@description $kind, $($variant),*) => $message.into(),
                    )*
                }
            }
        }
    };

    (@error $kind:path,) => {
        $kind
    };

    (@error $kind:path, $($variant:pat),+) => {
        $kind(..)
    };

    (@description $kind:path,) => {
        $kind
    };

    (@description $kind:path, $($variant:pat),+) => {
        $kind($($variant),*)
    };
}
