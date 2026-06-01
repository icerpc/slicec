// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel, Lint};
use crate::grammar::{attributes, Attributable, Entity};
use crate::slice_file::{SliceFile, Span};
use crate::slice_options::SliceOptions;
use serde::Serialize;

/// An annotated version of a [`Diagnostic`], whose [`DiagnosticLevel`] has been computed (taking into account any
/// 'allow' attributes or command-line flags), and that has pre-extracted text snippets to display alongside messages.
#[derive(Clone, Debug, Eq)]
pub struct AnnotatedDiagnostic {
    pub message: String,
    pub level: DiagnosticLevel,
    pub code: String,
    pub snippet: Option<Snippet>,
    pub notes: Vec<AnnotatedNote>,
    pub reported_by: Vec<String>,
}

impl PartialEq for AnnotatedDiagnostic {
    fn eq(&self, other: &Self) -> bool {
        // Check every field for equality except for 'reported_by'.
        // Who reported the diagnostic isn't an intrinsic property of the diagnostic itself.
        self.message == other.message
            && self.level == other.level
            && self.code == other.code
            && self.snippet == other.snippet
            && self.notes == other.notes
    }
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct AnnotatedNote {
    pub message: String,
    pub snippet: Option<Snippet>,
}

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Snippet {
    pub span: Span,
    pub text: String,
}

/// Creates an [`AnnotatedDiagnostic`] from the provided [`Diagnostic`].
pub fn convert_diagnostic(
    diagnostic: &Diagnostic,
    options: &SliceOptions,
    ast: &Ast,
    files: &[SliceFile],
) -> AnnotatedDiagnostic {
    let notes = diagnostic.notes.iter().map(|n| AnnotatedNote {
        message: n.message.clone(),
        snippet: get_snippet(&n.span, files),
    });

    // If the diagnostic was reported by a plugin, we just use the filename of the plugin (not its entire path).
    fn get_plugin_file_stem(plugin_path: &str) -> Option<&str> {
        std::path::Path::new(plugin_path).file_stem()?.to_str()
    }
    let reported_by = diagnostic.plugin.as_deref().map_or("slicec", |plugin_path| {
        get_plugin_file_stem(plugin_path).unwrap_or(plugin_path)
    });

    AnnotatedDiagnostic {
        message: diagnostic.message(),
        level: get_diagnostic_level_for(diagnostic, options, ast, files),
        code: diagnostic.code().to_owned(),
        snippet: get_snippet(&diagnostic.span, files),
        notes: notes.collect(),
        reported_by: vec![reported_by.to_owned()],
    }
}

/// Returns the [`DiagnosticLevel`] that the provided [`Diagnostic`] should be emitted with.
fn get_diagnostic_level_for(
    diagnostic: &Diagnostic,
    options: &SliceOptions,
    ast: &Ast,
    files: &[SliceFile],
) -> DiagnosticLevel {
    // Only lints can have their diagnostic levels changed (through attributes or command-line options).
    // For other kinds of diagnostics, we can immediately return their levels.
    let lint = match &diagnostic.kind {
        DiagnosticKind::Error(_) => return DiagnosticLevel::Error,
        DiagnosticKind::Info(_) => return DiagnosticLevel::Info,
        DiagnosticKind::Lint(lint) => lint,
    };

    // Helper function that checks whether a lint should be allowed according to the provided identifiers.
    fn is_lint_allowed_by<'b>(mut identifiers: impl Iterator<Item = &'b String>, lint: &Lint) -> bool {
        identifiers.any(|identifier| identifier == "All" || identifier == lint.lint_name())
    }

    // Helper function that checks whether a lint is allowed by attributes on the provided entity.
    fn is_lint_allowed_by_attributes(attributable: &(impl Attributable + ?Sized), lint: &Lint) -> bool {
        let attributes = attributable.all_attributes().into_iter();
        let mut allowed = attributes.filter_map(|a| a.downcast::<attributes::Allow>());
        allowed.any(|allow| is_lint_allowed_by(allow.allowed_lints.iter(), lint))
    }

    // Check if the lint is allowed by an `--allow` flag passed on the command line.
    if is_lint_allowed_by(options.allowed_lints.iter(), lint) {
        return DiagnosticLevel::Allowed;
    }

    // If the diagnostic has a span, check if it's affected by an `allow` attribute on its file.
    if let Some(span) = &diagnostic.span {
        let file = files.iter().find(|f| f.relative_path == span.file);
        if is_lint_allowed_by_attributes(file.unwrap(), lint) {
            return DiagnosticLevel::Allowed;
        }
    }

    // If the diagnostic has a scope, check if it's affected by an `allow` attribute in that scope.
    if let Some(scope) = &diagnostic.scope {
        if let Ok(entity) = ast.find_element::<dyn Entity>(scope) {
            if is_lint_allowed_by_attributes(entity, lint) {
                return DiagnosticLevel::Allowed;
            }
        }
    }

    // Otherwise, we just return the default diagnostic level for this lint.
    lint.default_diagnostic_level()
}

/// If `span` is `Some`, this tries to extract a text snippet corresponding to the file & locations contained in the
/// span. If `span` is `None` or if the text couldn't be extracted, this returns `None`.
fn get_snippet(span: &Option<Span>, files: &[SliceFile]) -> Option<Snippet> {
    let span = span.clone()?;
    let snippet_file = files.iter().find(|file| file.relative_path == span.file)?;
    let text = snippet_file.get_snippet(span.start, span.end);
    Some(Snippet { span, text })
}
