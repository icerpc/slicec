// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Span;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;

mod diagnostic_reporter;
mod logic;
mod warnings;

pub use self::diagnostic_reporter::DiagnosticReporter;
pub use self::logic::LogicErrorKind;
pub use self::warnings::WarningKind;

/// A Diagnostic contains information about syntax errors, logic errors, etc., encountered while compiling slice
/// code.
///
/// Each Diagnostic has a kind, specifying the type of diagnostic encountered, such as SyntaxError, LogicError, or IO.
/// Additionally, a Diagnostic can have an optional Span which specifies the location in the source code where the
/// diagnostic occurred.
#[derive(Debug)]
pub struct Diagnostic {
    pub diagnostic_kind: DiagnosticKind,
    pub span: Option<Span>,
    pub notes: Vec<Note>,
}

impl Diagnostic {
    pub fn new(diagnostic_kind: impl Into<DiagnosticKind>, span: Option<&Span>) -> Self {
        Diagnostic {
            diagnostic_kind: diagnostic_kind.into(),
            span: span.cloned(),
            notes: Vec::new(),
        }
    }

    pub fn new_with_notes(diagnostic_kind: impl Into<DiagnosticKind>, span: Option<&Span>, notes: Vec<Note>) -> Self {
        Diagnostic {
            diagnostic_kind: diagnostic_kind.into(),
            span: span.cloned(),
            notes,
        }
    }

    pub fn attach_notes(&mut self, notes: Vec<Note>) {
        self.notes.extend(notes);
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.diagnostic_kind)
    }
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Diagnostic", 4)?;
        state.serialize_field("message", &self.diagnostic_kind.to_string())?;
        state.serialize_field("severity", &self.diagnostic_kind)?;
        state.serialize_field("span", &self.span)?;
        state.serialize_field("notes", &self.notes)?;
        state.end()
    }
}

#[derive(Debug)]
pub enum DiagnosticKind {
    /// An error related to the syntax of the slice source code such as missing semicolons or defining classes in a
    /// Slice2 encoded slice file.
    SyntaxError(String),

    /// An error related to the logic of the slice source code such as using the same tag twice.
    LogicError(LogicErrorKind),

    /// A suggestion or warning to aid in preventing a problem. For example warning if a documentation comment
    /// indicates that an operation should return a value, but the operation does not.
    Warning(WarningKind),

    /// An error related to the IO of the slice source code such as opening a file that doesn't exist.
    IOError(String),
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiagnosticKind::SyntaxError(error) => write!(f, "{}", error),
            DiagnosticKind::LogicError(rule_kind) => write!(f, "{}", rule_kind.message()),
            DiagnosticKind::Warning(warning_kind) => write!(f, "{}", warning_kind.message()),
            DiagnosticKind::IOError(error) => write!(f, "{}", error),
        }
    }
}

impl Serialize for DiagnosticKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind = match self {
            DiagnosticKind::Warning(_) => "warning",
            DiagnosticKind::LogicError(_) | DiagnosticKind::SyntaxError(_) | DiagnosticKind::IOError(_) => "error",
        };
        serializer.serialize_str(kind)
    }
}

/// Additional information about a diagnostic. For example, indicating where the encoding of a Slice1 encoded Slice file
/// was defined.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

impl Note {
    pub fn new(message: impl Into<String>, span: Option<&Span>) -> Self {
        Note {
            message: message.into(),
            span: span.cloned(),
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
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
