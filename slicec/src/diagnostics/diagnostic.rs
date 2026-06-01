// Copyright (c) ZeroC, Inc.

use super::{DiagnosticKind, Error, Lint, Note};
use crate::slice_file::Span;

/// A message that is reported by the compiler to provide information and context for errors, warnings, or other issues.
#[derive(Debug)]
pub struct Diagnostic {
    /// The exact kind of diagnostic.
    pub kind: DiagnosticKind,

    /// The section of a Slice file that this diagnostic is about.
    /// If present, a snippet of text from this section will be reported alongside the diagnostic.
    pub span: Option<Span>,

    /// The Slice scope that this diagnostic was emitted from (used to check for 'allow' metadata).
    pub scope: Option<String>,

    /// Any additional information that should be reported alongside this diagnostic's main message.
    pub notes: Vec<Note>,

    /// The plugin that reported this diagnostic, or `None` if this diagnostic was reported by 'slicec' itself.
    pub plugin: Option<String>,
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
            plugin: None,
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

    /// Creates a new informational `Diagnostic` from the provided message.
    /// The newly created `Diagnostic` has no `span`, `scope`, or `notes` set.
    pub fn from_info(message: impl Into<String>) -> Self {
        Self::new(DiagnosticKind::Info(message.into()))
    }

    /// Returns this diagnostic's message.
    pub fn message(&self) -> String {
        match &self.kind {
            DiagnosticKind::Error(error) => error.message(),
            DiagnosticKind::Lint(lint) => lint.message(),
            DiagnosticKind::Info(message) => message.clone(),
        }
    }

    /// Returns this diagnostic's code, unless it's an informational diagnostic, in which case this returns `""`.
    /// For errors, this is of the form `E###`; for lints this is the name of the lint;
    pub fn code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.error_code(),
            DiagnosticKind::Lint(lint) => lint.lint_name(),
            DiagnosticKind::Info(_) => "",
        }
    }

    /// Sets this diagnostic's span to the provided value.
    pub fn set_span(mut self, span: &Span) -> Self {
        assert!(self.span.is_none()); // Calling this function multiple times is probably a bug on our part.
        self.span = Some(span.to_owned());
        self
    }

    /// Sets this diagnostic's scope to the provided value.
    pub fn set_scope(mut self, scope: impl Into<String>) -> Self {
        assert!(self.scope.is_none()); // Calling this function multiple times is probably a bug on our part.
        self.scope = Some(scope.into());
        self
    }

    /// Adds a [`Note`] to this diagnostic.
    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note {
            message: message.into(),
            span: span.cloned(),
        });
        self
    }

    /// Moves this diagnostic into the provided [`Diagnostics`] collection.
    pub fn push_into(self, diagnostics: &mut Diagnostics) {
        diagnostics.0.push(self);
    }
}

/// Stores a collection of diagnostics.
#[derive(Debug, Default)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Creates a new (empty) collection of diagnostics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this contains any diagnostics that are errors.
    pub fn has_errors(&self) -> bool {
        let mut diagnostics = self.0.iter();
        diagnostics.any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::Error(_)))
    }

    /// Consumes this [`Diagnostics`] and returns its inner collection of [`Diagnostic`]s.
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
