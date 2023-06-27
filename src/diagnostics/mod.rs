// Copyright (c) ZeroC, Inc.

use crate::slice_file::Span;
use serde::Serialize;

mod diagnostic_reporter;
mod errors;
mod lints;

pub use diagnostic_reporter::DiagnosticReporter;
pub use errors::Error;
pub use lints::Lint;

/// A diagnostic is a message that is reported to the user during compilation.
/// It can either hold an [Error] or a [Lint].
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
            DiagnosticKind::Lint(lint) => lint.message(),
        }
    }

    /// Returns the error code of this diagnostic if it has one.
    pub fn error_code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.error_code(),
            DiagnosticKind::Lint(lint) => lint.error_code(),
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

#[derive(Debug)]
pub enum DiagnosticKind {
    Error(Error),
    Lint(Lint),
}

impl From<Error> for DiagnosticKind {
    fn from(error: Error) -> Self {
        DiagnosticKind::Error(error)
    }
}

impl From<Lint> for DiagnosticKind {
    fn from(lint: Lint) -> Self {
        DiagnosticKind::Lint(lint)
    }
}

/// Additional information about a diagnostic.
/// For example, indicating where the encoding of a Slice1 encoded Slice file was defined.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

/// A macro that implements the `error_code` and `message` functions for [Lint] and [Error] enums.
#[macro_export]
macro_rules! implement_diagnostic_functions {
    (Lint, $(($kind:ident, $message:expr $(, $variant:ident)* )),*) => {
        impl Lint {
            // TODO maybe we should move this somewhere other than `Lint`? Like in `Attribute` maybe?
            /// This array contains all the valid arguments for the 'allow' attribute.
            pub const ALLOWABLE_LINT_IDENTIFIERS: [&str; 6] = [
                "All",
                $(stringify!($kind)),*
            ];

            pub fn error_code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error Lint::$kind, $($variant),*) => stringify!($kind),
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_diagnostic_functions!(@description Lint::$kind, $($variant),*) => $message.into(),
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
