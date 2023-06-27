// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, DiagnosticKind, Lint};
use crate::grammar::{attributes, Attributable, Entity};
use crate::slice_file::SliceFile;
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    diagnostics: Vec<Diagnostic>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
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
            error_count: 0,
            warning_count: 0,
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_lints: slice_options.allowed_lints.clone(),
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        self.error_count != 0
    }

    /// Checks if any diagnostics have been reported during compilation so far.
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Returns 1 if any errors were reported and 0 if no errors were reported.
    pub fn get_exit_code(&self) -> i32 {
        i32::from(self.has_errors())
    }

    /// Consumes the diagnostic reporter and returns an iterator over its diagnostics, with any suppressed warnings
    /// filtered out. (ie: any warnings covered by an `allow` attribute or a `--allow` command line flag).
    pub fn into_diagnostics<'a>(
        self,
        ast: &'a Ast,
        files: &'a HashMap<String, SliceFile>,
    ) -> impl Iterator<Item = Diagnostic> + 'a {
        // Helper function that checks whether a warning should be suppressed according to the provided identifiers.
        fn is_warning_suppressed_by<'b>(mut identifiers: impl Iterator<Item = &'b String>, warning: &Lint) -> bool {
            identifiers.any(|identifier| identifier == "All" || identifier == warning.error_code())
        }

        // Helper function that checks whether a warning is suppressed by attributes on the provided entity.
        fn is_warning_suppressed_by_attributes(attributable: &(impl Attributable + ?Sized), warning: &Lint) -> bool {
            let attributes = attributable.all_attributes().concat().into_iter();
            let mut allowed = attributes.filter_map(|a| a.downcast::<attributes::Allow>());
            allowed.any(|allow| is_warning_suppressed_by(allow.allowed_lints.iter(), warning))
        }

        // Filter out any diagnostics that should be suppressed.
        self.diagnostics.into_iter().filter(move |diagnostic| {
            let mut is_suppressed = false;

            if let DiagnosticKind::Lint(warning) = &diagnostic.kind {
                // Check if the warning is suppressed by an `--allow` flag passed on the command line.
                is_suppressed |= is_warning_suppressed_by(self.allowed_lints.iter(), warning);

                // If the warning has a span, check if it's suppressed by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.get(&span.file).expect("slice file didn't exist");
                    is_suppressed |= is_warning_suppressed_by_attributes(file, warning);
                }

                // If the warning has a scope, check if it's suppressed by an `allow` attribute in that scope.
                if let Some(scope) = diagnostic.scope() {
                    if let Ok(entity) = ast.find_element::<dyn Entity>(scope) {
                        is_suppressed |= is_warning_suppressed_by_attributes(entity, warning);
                    }
                }
            }
            !is_suppressed
        })
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        match &diagnostic.kind {
            DiagnosticKind::Error(_) => self.error_count += 1,
            DiagnosticKind::Lint(_) => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}
