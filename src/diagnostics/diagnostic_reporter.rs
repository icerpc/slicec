// Copyright (c) ZeroC, Inc.

use super::{Diagnostic, DiagnosticKind, DiagnosticLevel, Lint};
use crate::ast::Ast;
use crate::grammar::{attributes, Attributable, Entity};
use crate::slice_file::SliceFile;
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    pub diagnostics: Vec<Diagnostic>,
    /// Lists all the lints that should be allowed by this reporter.
    pub allowed_lints: Vec<String>,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    /// If true, diagnostic output will not be styled with colors.
    pub disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_lints: slice_options.allowed_lints.clone(),
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        let mut diagnostics = self.diagnostics.iter();
        diagnostics.any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::Error(_)))
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    // TODO COMMENT
    // TODO add support for deny/warn lint configuration attributes & command line options.
    pub fn update_diagnostics(&mut self, ast: &Ast, files: &HashMap<String, SliceFile>) -> (usize, usize) {
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

        // TODO COMMENT
        let (mut total_warnings, mut total_errors) = (0, 0);
        for diagnostic in &mut self.diagnostics {
            // If this diagnostic is a lint, update its diagnostic level. Errors always have a level of `Error`.
            if let DiagnosticKind::Lint(lint) = &diagnostic.kind {
                // Check if the lint is allowed by an `--allow` flag passed on the command line.
                if is_lint_allowed_by(self.allowed_lints.iter(), lint) {
                    diagnostic.level = DiagnosticLevel::Allowed;
                }

                // If the diagnostic has a span, check if it's affected by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.get(&span.file).expect("slice file didn't exist");
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

            // Update the total number of errors/warnings accordingly.
            match diagnostic.level() {
                DiagnosticLevel::Error => total_errors += 1,
                DiagnosticLevel::Warning => total_warnings += 1,
                DiagnosticLevel::Allowed => {}
            }
        }

        (total_warnings, total_errors)
    }
}
