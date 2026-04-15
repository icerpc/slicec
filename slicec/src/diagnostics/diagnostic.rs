// Copyright (c) ZeroC, Inc.

use super::{DiagnosticKind, Error, Lint, Note};
use crate::slice_file::Span;

/// A diagnostic is a message that is reported to the user during compilation.
#[derive(Debug)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Option<Span>,
    pub scope: Option<String>,
    pub notes: Vec<Note>,
}

impl Diagnostic {
    /// Creates a new `Diagnostic` directly from a [`DiagnosticKind`].
    /// The newly created `Diagnostic` has no `span`, `scope`, or `notes` set.
    pub fn new(kind: DiagnosticKind) -> Self {
        Diagnostic {
            kind,
            span: None,
            scope: None,
            notes: Vec::new(),
        }
    }

    /// Creates a new error `Diagnostic` from the provided [`Error`].
    /// The newly created `Diagnostic` has no `span`, `scope`, or `notes` set.
    pub fn from_error(error: Error) -> Self {
        Self::new(DiagnosticKind::Error(error))
    }

    /// Creates a new lint `Diagnostic` from the provided [`Lint`].
    /// The newly created `Diagnostic` has no `span`, `scope`, or `notes` set.
    pub fn from_lint(lint: Lint) -> Self {
        Self::new(DiagnosticKind::Lint(lint))
    }

    /// Returns the message of this diagnostic.
    pub fn message(&self) -> String {
        match &self.kind {
            DiagnosticKind::Error(error) => error.message(),
            DiagnosticKind::Lint(lint) => lint.message(),
        }
    }

    /// Returns this diagnostic's code. This is either the name of a lint or of the form `E###`.
    pub fn code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.error_code(),
            DiagnosticKind::Lint(lint) => lint.lint_name(),
        }
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

    pub fn push_into(self, diagnostics: &mut Diagnostics) {
        diagnostics.0.push(self);
    }
}

#[derive(Debug, Default)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Creates a new diagnostics container that is empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this contains any diagnostics that are errors.
    pub fn has_errors(&self) -> bool {
        let mut diagnostics = self.0.iter();
        diagnostics.any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::Error(_)))
    }

    /// Returns the diagnostics held by this without any updates or patches.
    /// This should only be called by tests that want to bypass this behavior.
    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.0
    }
}

impl std::ops::Deref for Diagnostics {
    type Target = Vec<Diagnostic>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Diagnostics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
