// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Span;
use std::fmt;

mod diagnostic_reporter;
mod logic;
mod warnings;

pub use self::diagnostic_reporter::DiagnosticReporter;
pub use self::logic::LogicKind;
pub use self::warnings::WarningKind;

/// A Diagnostic contains information about syntax errors, logic errors, etc., encountered while compiling slice code.
///
/// Each Diagnostic has a kind, specifying the type of diagnostic encountered, such as SyntaxError, LogicError, or
/// IO. Additionally, a Diagnostic can have an optional Span which specifies the location in the source code where the
/// diagnostic occurred.
#[derive(Debug)]
pub struct Diagnostic {
    pub error_kind: DiagnosticKind,
    pub span: Option<Span>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind)
    }
}

#[derive(Debug)]
pub enum DiagnosticKind {
    /// An error related to the syntax of the slice source code such as missing semicolons or defining classes in a
    /// Slice2 encoded slice file.
    SyntaxError(String),

    /// An error related to the logic of the slice source code such as using the same tag twice.
    LogicError(LogicKind),

    /// A suggestion or warning to aid in preventing a problem. For example warning if a documentation comment
    /// indicates that an operation should return a value, but the operation does not.
    Warning(WarningKind),

    /// Additional information about another kind of error that was encountered. For example, indicating where the
    /// encoding of a Slice1 encoded slice file was defined.
    Note(String),

    /// An error related to the IO of the slice source code such as opening a file that doesn't exist.
    IOError(String),
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiagnosticKind::SyntaxError(error) => write!(f, "{}", error),
            DiagnosticKind::LogicError(rule_kind) => write!(f, "{}", rule_kind.message()),
            DiagnosticKind::Warning(warning_kind) => write!(f, "{}", warning_kind.message()),
            DiagnosticKind::Note(note) => write!(f, "{}", note),
            DiagnosticKind::IOError(error) => write!(f, "{}", error),
        }
    }
}

/// Creates a new note from a string.
///
/// # Examples
/// ```
/// # use slice::diagnostics::DiagnosticKind;
/// let note = DiagnosticKind::new_note("This is the content of a note.");
/// ```
impl DiagnosticKind {
    pub fn new_note(message: impl Into<String>) -> DiagnosticKind {
        DiagnosticKind::Note(message.into())
    }
}

#[macro_export]
macro_rules! implement_from_for_error_sub_kind {
    ($type:ty, $enumerator:path) => {
        impl From<$type> for DiagnosticKind {
            fn from(original: $type) -> DiagnosticKind {
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
