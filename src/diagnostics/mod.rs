// Copyright (c) ZeroC, Inc.

use crate::slice_file::Span;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

mod diagnostic_reporter;
mod errors;
mod warnings;

pub use diagnostic_reporter::DiagnosticReporter;
pub use errors::Error;
pub use warnings::Warning;

/// A diagnostic is a message that is reported to the user during compilation.
/// It can either hold an [Error] or a [Warning].
#[derive(Debug)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    span: Option<Span>,
    scope: Option<String>,
    notes: Vec<Note>,
}

impl Diagnostic {
    pub fn new(kind: impl Into<DiagnosticKind>) -> Self {
        Diagnostic {
            kind: kind.into(),
            span: None,
            scope: None,
            notes: Vec::new(),
        }
    }

    /// Returns the message of this diagnostic.
    pub fn message(&self) -> String {
        match &self.kind {
            DiagnosticKind::Error(error) => error.message(),
            DiagnosticKind::Warning(warning) => warning.message(),
        }
    }

    /// Returns the error code of this diagnostic if it has one.
    pub fn error_code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.error_code(),
            DiagnosticKind::Warning(warning) => warning.error_code(),
        }
    }

    /// Returns the [Span] of this diagnostic if it has one.
    pub fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    /// Returns the scope of this diagnostic if it has one.
    pub fn scope(&self) -> Option<&String> {
        self.scope.as_ref()
    }

    /// Returns any [Notes](Note) associated with this diagnostic.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn set_span(mut self, span: &Span) -> Self {
        self.span = Some(span.to_owned());
        self
    }

    pub fn set_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note {
            message: message.into(),
            span: span.cloned(),
        });
        self
    }

    pub fn extend_notes<I: IntoIterator<Item = Note>>(mut self, iter: I) -> Self {
        self.notes.extend(iter);
        self
    }

    pub fn report(self, diagnostic_reporter: &mut DiagnosticReporter) {
        diagnostic_reporter.report(self);
    }
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Diagnostic", 5)?;
        let severity = match &self.kind {
            DiagnosticKind::Error(_) => "error",
            DiagnosticKind::Warning(_) => "warning",
        };
        state.serialize_field("message", &self.message())?;
        state.serialize_field("severity", severity)?;
        state.serialize_field("span", &self.span())?;
        state.serialize_field("notes", self.notes())?;
        state.serialize_field("error_code", self.error_code())?;
        state.end()
    }
}

#[derive(Debug)]
pub enum DiagnosticKind {
    Error(Error),
    Warning(Warning),
}

impl From<Error> for DiagnosticKind {
    fn from(error: Error) -> Self {
        DiagnosticKind::Error(error)
    }
}

impl From<Warning> for DiagnosticKind {
    fn from(warning: Warning) -> Self {
        DiagnosticKind::Warning(warning)
    }
}

/// Additional information about a diagnostic.
/// For example, indicating where the encoding of a Slice1 encoded Slice file was defined.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

/// A macro that implements the `error_code` and `message` functions for [Warning] and [Error] enums.
#[macro_export]
macro_rules! implement_diagnostic_functions {
    (Warning, $(($kind:ident, $message:expr $(, $variant:ident)* )),*) => {
        impl Warning {
            // TODO maybe we should move this somewhere other than `Warning`? Like in `Attribute` maybe?
            /// This array contains all the valid arguments for the 'allow' attribute.
            pub const ALLOWABLE_WARNING_IDENTIFIERS: [&str; 6] = [
                "All",
                $(stringify!($kind)),*
            ];

            pub fn error_code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error Warning::$kind, $($variant),*) => stringify!($kind),
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_diagnostic_functions!(@description Warning::$kind, $($variant),*) => $message.into(),
                    )*
                }
            }
        }
    };

    (Error, $(($code:literal, $kind:ident, $message:expr $(, $variant:ident)* )),*) => {
        impl Error {
            pub fn error_code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error Error::$kind, $($variant),*) => $code,
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_diagnostic_functions!(@description Error::$kind, $($variant),*) => $message.into(),
                    )*
                }
            }
        }
    };

    (@error $kind:path,) => {
        $kind
    };

    (@error $kind:path, $($variant:ident),+) => {
        $kind {..}
    };

    (@description $kind:path,) => {
        $kind
    };

    (@description $kind:path, $($variant:ident),+) => {
        $kind{$($variant),*}
    };
}
