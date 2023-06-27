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
    pub(self) level: DiagnosticLevel,
    span: Option<Span>,
    scope: Option<String>,
    notes: Vec<Note>,
}

impl Diagnostic {
    pub fn new(kind: impl Into<DiagnosticKind>) -> Self {
        let kind = kind.into();
        let level = match &kind {
            DiagnosticKind::Error(_) => DiagnosticLevel::Error,
            DiagnosticKind::Lint(lint) => lint.get_default_level(),
        };

        Diagnostic {
            kind,
            level,
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

    /// Returns this diagnostic's code. This is either the name of a lint, or of the form `E###`.
    pub fn code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.code(),
            DiagnosticKind::Lint(lint) => lint.code(),
        }
    }

    /// Returns the [level](Level) of this diagnostic.
    /// Note that this value may change after the diagnostic is reported, since levels can be changed by attributes.
    pub fn level(&self) -> DiagnosticLevel {
        self.level
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

/// Diagnostic levels describe the severity of a diagnostic, and how the compiler should react to their emission.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    #[rustfmt::skip] // See https://github.com/rust-lang/rustfmt/issues/5801
    /// Diagnostics with the `Error` level will be emitted and will cause compilation to fail with a non-zero exit code.
    Error,

    /// Diagnostics with the `Warning` level will be emitted, but will not influence the exit code of the compiler.
    Warning,

    /// Diagnostics with the `Allowed` level will be suppressed and will not emit any message.
    Allowed,
}

/// A macro that implements the `code` and `message` functions for [Lint] and [Error] enums.
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

            pub fn code(&self) -> &str {
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
            pub fn code(&self) -> &str {
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
