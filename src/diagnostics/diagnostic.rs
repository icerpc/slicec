// Copyright (c) ZeroC, Inc.

use super::{Error, Lint, Note};
use crate::ast::Ast;
use crate::grammar::{attributes, Attributable, Entity};
use crate::slice_file::{SliceFile, Span};
use crate::slice_options::SliceOptions;

/// A diagnostic is a message that is reported to the user during compilation.
/// It can either hold an [Error] or a [Lint].
#[derive(Debug)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    level: DiagnosticLevel,
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

    /// Returns this diagnostic's code. This is either the name of a lint or of the form `E###`.
    pub fn code(&self) -> &str {
        match &self.kind {
            DiagnosticKind::Error(error) => error.code(),
            DiagnosticKind::Lint(lint) => lint.code(),
        }
    }

    /// Returns the [level](DiagnosticLevel) of this diagnostic.
    /// Note that this value may change after the diagnostic is reported, since levels can be altered by attributes.
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

    pub fn push_into(self, diagnostics: &mut Diagnostics) {
        diagnostics.0.push(self);
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

#[derive(Debug, Default)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Creates a new diagnostics container that is empty.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.0.extend(other.0);
    }

    /// Returns true if this contains any diagnostics that are errors.
    pub fn has_errors(&self) -> bool {
        let mut diagnostics = self.0.iter();
        diagnostics.any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::Error(_)))
    }

    /// Returns true if this contains no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the diagnostics this struct contains after it has patched and updated them.
    /// Lint levels can be configured via attributes or command line options, but these aren't applied until this runs.
    pub fn into_updated(mut self, ast: &Ast, files: &[SliceFile], options: &SliceOptions) -> Vec<Diagnostic> {
        // Helper function that checks whether a lint should be allowed according to the provided identifiers.
        fn is_lint_allowed_by<'b>(mut identifiers: impl Iterator<Item = &'b String>, lint: &Lint) -> bool {
            identifiers.any(|identifier| identifier == "All" || identifier == lint.code())
        }

        // Helper function that checks whether a lint is allowed by attributes on the provided entity.
        fn is_lint_allowed_by_attributes(attributable: &(impl Attributable + ?Sized), lint: &Lint) -> bool {
            let attributes = attributable.all_attributes().concat().into_iter();
            let mut allowed = attributes.filter_map(|a| a.downcast::<attributes::Allow>());
            allowed.any(|allow| is_lint_allowed_by(allow.allowed_lints.iter(), lint))
        }

        for diagnostic in &mut self.0 {
            // If this diagnostic is a lint, update its diagnostic level. Errors always have a level of `Error`.
            if let DiagnosticKind::Lint(lint) = &diagnostic.kind {
                // Check if the lint is allowed by an `--allow` flag passed on the command line.
                if is_lint_allowed_by(options.allowed_lints.iter(), lint) {
                    diagnostic.level = DiagnosticLevel::Allowed;
                }

                // If the diagnostic has a span, check if it's affected by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.iter().find(|f| f.relative_path == span.file).expect("no file");
                    if is_lint_allowed_by_attributes(file, lint) {
                        diagnostic.level = DiagnosticLevel::Allowed;
                    }
                }

                // If the diagnostic has a scope, check if it's affected by an `allow` attribute in that scope.
                if let Some(scope) = diagnostic.scope() {
                    if let Ok(entity) = ast.find_element::<dyn Entity>(scope) {
                        if is_lint_allowed_by_attributes(entity, lint) {
                            diagnostic.level = DiagnosticLevel::Allowed;
                        }
                    }
                }
            }
        }
        self.0
    }

    /// Returns the diagnostics held by this without any updates or patches.
    /// This should only be called by tests that want to bypass this behavior.
    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.0
    }
}

pub fn get_totals(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let (mut total_warnings, mut total_errors) = (0, 0);

    for diagnostic in diagnostics {
        match diagnostic.level() {
            DiagnosticLevel::Error => total_errors += 1,
            DiagnosticLevel::Warning => total_warnings += 1,
            DiagnosticLevel::Allowed => {}
        }
    }

    (total_warnings, total_errors)
}
