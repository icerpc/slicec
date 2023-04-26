// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, DiagnosticKind, Warning};
use crate::grammar::{validate_allow_arguments, Attributable, Attribute, Entity};
use crate::slice_file::SliceFile;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    diagnostics: Vec<Diagnostic>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
    /// If true, compilation will fail on warnings in addition to errors.
    treat_warnings_as_errors: bool,
    /// Lists all the warnings that should be suppressed by this reporter.
    pub allowed_warnings: Vec<String>,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    /// If true, diagnostic output will not be styled.
    pub disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        let mut diagnostic_reporter = DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors: slice_options.warn_as_error,
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_warnings: slice_options.allowed_warnings.clone(),
        };

        // Validate any arguments for `--allowed-warnings` that were passed into the command line.
        validate_allow_arguments(&slice_options.allowed_warnings, None, &mut diagnostic_reporter);

        diagnostic_reporter
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        self.error_count != 0
    }

    /// Checks if any diagnostics (warnings or errors) have been reported during compilation.
    pub fn has_diagnostics(&self) -> bool {
        self.error_count + self.warning_count != 0
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Returns 1 if any errors were reported and 0 if no errors were reported.
    /// If `treat_warnings_as_errors` is true, warnings well be counted as errors by this function.
    pub fn get_exit_code(&self) -> i32 {
        i32::from(self.has_errors() || (self.treat_warnings_as_errors && self.has_diagnostics()))
    }

    /// Consumes the diagnostic reporter and returns an iterator over its diagnostics, with any suppressed warnings
    /// filtered out (ie: any warnings specified by `allow` attributes, or the `--allow-warnings` command line option).
    pub fn into_diagnostics<'a>(
        self,
        ast: &'a Ast,
        files: &'a HashMap<String, SliceFile>,
    ) -> impl Iterator<Item = Diagnostic> + 'a {
        // Helper function that checks whether a warning should be suppressed according to the provided identifiers.
        fn is_warning_suppressed_by<'b>(mut identifiers: impl Iterator<Item = &'b String>, warning: &Warning) -> bool {
            identifiers.any(|identifier| identifier == "All" || identifier == warning.error_code())
        }

        // Helper function that checks whether a warning should be suppressed according to the provided attributes.
        fn is_warning_suppressed_by_attributes(attributes: Vec<&Attribute>, warning: &Warning) -> bool {
            let mut allowed_warnings = attributes.into_iter().filter_map(Attribute::match_allow_warnings);
            allowed_warnings.any(|allowed| is_warning_suppressed_by(allowed.iter(), warning))
        }

        // Filter out any diagnostics that should be suppressed.
        self.diagnostics.into_iter().filter(move |diagnostic| {
            let mut is_suppressed = false;

            if let DiagnosticKind::Warning(warning) = &diagnostic.kind {
                // Check if the warning is allowed by an `allowed-warnings` flag passed on the command line.
                is_suppressed |= is_warning_suppressed_by(self.allowed_warnings.iter(), warning);

                // If the warning has a span, check if it's allowed by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.get(&span.file).expect("slice file didn't exist");
                    is_suppressed |= is_warning_suppressed_by_attributes(file.attributes(false), warning);
                }

                // If the warning has a scope, check if it's allowed by an `allow` attribute in that scope.
                if let Some(scope) = diagnostic.scope() {
                    let entity = ast.find_element::<dyn Entity>(scope).expect("entity didn't exist");
                    is_suppressed |= is_warning_suppressed_by_attributes(entity.attributes(true), warning);
                }
            }
            !is_suppressed
        })
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        match &diagnostic.kind {
            DiagnosticKind::Error(_) => self.error_count += 1,
            DiagnosticKind::Warning(_) => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}
