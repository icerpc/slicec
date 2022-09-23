// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Span;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;

mod diagnostic_reporter;
mod errors;
mod warnings;

pub use self::diagnostic_reporter::DiagnosticReporter;
pub use self::errors::ErrorKind;
pub use self::warnings::WarningKind;

/// A Diagnostic contains information about syntax errors, logic errors, etc., encountered while compiling slice
/// code.
///
/// Each Diagnostic has a kind, specifying the type of diagnostic encountered, such as SyntaxError, LogicError, or IO.
/// Additionally, a Diagnostic can have an optional Span which specifies the location in the source code where the
/// diagnostic occurred.
#[derive(Debug)]
pub enum Diagnostic {
    Error(Error),
    Warning(Warning),
}

impl Diagnostic {
    pub fn message(&self) -> String {
        match self {
            Diagnostic::Error(error) => error.to_string(),
            Diagnostic::Warning(warning) => warning.to_string(),
        }
    }

    pub fn span(&self) -> Option<&Span> {
        match self {
            Diagnostic::Error(error) => error.span.as_ref(),
            Diagnostic::Warning(warning) => Some(&warning.span),
        }
    }

    pub fn notes(&self) -> &Vec<Note> {
        match self {
            Diagnostic::Error(error) => &error.notes,
            Diagnostic::Warning(warning) => &warning.notes,
        }
    }

    pub fn error_code(&self) -> Option<&str> {
        match self {
            Diagnostic::Error(error) => error.error_code(),
            Diagnostic::Warning(warning) => Some(warning.error_code()),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Diagnostic", 4)?;
        let severity = match &self {
            Diagnostic::Error(_) => "error",
            Diagnostic::Warning(_) => "warning",
        };
        state.serialize_field("message", &self.message())?;
        state.serialize_field("severity", severity)?;
        state.serialize_field("span", &self.span())?;
        state.serialize_field("notes", &self.notes())?;
        state.serialize_field("error_code", &self.error_code())?;
        state.end()
    }
}

#[derive(Debug)]
pub struct Warning {
    kind: WarningKind,
    span: Span,
    notes: Vec<Note>,
}

impl Warning {
    pub fn new(warning_kind: WarningKind, span: &Span) -> Self {
        Warning {
            kind: warning_kind,
            span: span.clone(),
            notes: Vec::new(),
        }
    }

    pub fn new_with_notes(warning_kind: WarningKind, span: &Span, notes: Vec<Note>) -> Self {
        Warning {
            kind: warning_kind,
            span: span.clone(),
            notes,
        }
    }

    pub fn attach_notes(&mut self, notes: Vec<Note>) {
        self.notes.extend(notes);
    }

    pub fn error_code(&self) -> &str {
        self.kind.error_code()
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind.message())
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    span: Option<Span>,
    notes: Vec<Note>,
}

impl Error {
    pub fn new(error_kind: impl Into<ErrorKind>, span: Option<&Span>) -> Self {
        Error {
            kind: error_kind.into(),
            span: span.cloned(),
            notes: Vec::new(),
        }
    }

    pub fn new_with_notes(error_kind: impl Into<ErrorKind>, span: Option<&Span>, notes: Vec<Note>) -> Self {
        Error {
            kind: error_kind.into(),
            span: span.cloned(),
            notes,
        }
    }

    pub fn attach_notes(&mut self, notes: Vec<Note>) {
        self.notes.extend(notes);
    }

    pub fn error_code(&self) -> Option<&str> {
        self.kind.error_code()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind.message())
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
macro_rules! implement_error_functions {
    (WarningKind, $(($code:expr, $kind:path, $message:expr $(, $variant:pat)* )),*) => {
        impl WarningKind {
            pub fn error_code(&self) -> &str {
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

    (ErrorKind, $(($($code:literal,)? $kind:path, $message:expr $(, $variant:pat)* )),*) => {
        impl ErrorKind {
            pub fn error_code(&self) -> Option<&str> {
                match self {
                    $(
                        implement_error_functions!(@error $kind, $($variant),*) => implement_error_functions!(@code $($code)?),
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

    (@code $code:literal) => {
        Some($code)
    };

    (@code) => {
        None
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
