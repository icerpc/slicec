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

    pub fn span(&self) -> &Option<Span> {
        match self {
            Diagnostic::Error(kind) => &kind.span,
            Diagnostic::Warning(kind) => &kind.span,
        }
    }

    pub fn notes(&self) -> &Vec<Note> {
        match self {
            Diagnostic::Error(kind) => &kind.notes,
            Diagnostic::Warning(kind) => &kind.notes,
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
        let message = &self.message();
        let severity = match &self {
            Diagnostic::Error(_) => "error",
            Diagnostic::Warning(_) => "warning",
        };
        state.serialize_field("message", message)?;
        state.serialize_field("severity", severity)?;
        state.serialize_field("span", &self.span())?;
        state.serialize_field("notes", &self.notes())?;
        state.end()
    }
}

#[derive(Debug)]
pub struct Warning {
    kind: WarningKind,
    span: Option<Span>,
    notes: Vec<Note>,
}

impl Warning {
    pub fn new(warning_kind: WarningKind, span: Option<&Span>) -> Self {
        Warning {
            kind: warning_kind,
            span: span.cloned(),
            notes: Vec::new(),
        }
    }

    pub fn new_with_notes(warning_kind: WarningKind, span: Option<&Span>, notes: Vec<Note>) -> Self {
        Warning {
            kind: warning_kind,
            span: span.cloned(),
            notes,
        }
    }

    pub fn attach_notes(&mut self, notes: Vec<Note>) {
        self.notes.extend(notes);
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.kind.message())
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
        let error_kind: ErrorKind = error_kind.into();
        Error {
            kind: error_kind,
            span: span.cloned(),
            notes: Vec::new(),
        }
    }

    pub fn new_with_notes(error_kind: impl Into<ErrorKind>, span: Option<&Span>, notes: Vec<Note>) -> Self {
        let error_kind: ErrorKind = error_kind.into();
        Error {
            kind: error_kind,
            span: span.cloned(),
            notes,
        }
    }

    pub fn attach_notes(&mut self, notes: Vec<Note>) {
        self.notes.extend(notes);
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.kind)
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

#[derive(Debug)]
pub enum ErrorKind {
    /// An error related to the syntax of the slice source code such as missing semicolons or defining classes in a
    /// Slice2 encoded slice file.
    Syntax(String),

    /// An error related to the logic of the slice source code such as using the same tag twice.
    Logic(LogicErrorKind),

    /// An error related to the IO of the slice source code such as opening a file that doesn't exist.
    IO(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ErrorKind::Syntax(message) => write!(f, "{}", message),
            ErrorKind::Logic(logic_error_kind) => write!(f, "{}", logic_error_kind.message()),
            ErrorKind::IO(message) => write!(f, "{}", message),
        }
    }
}

impl From<LogicErrorKind> for ErrorKind {
    fn from(original: LogicErrorKind) -> Self {
        Self::Logic(original)
    }
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
