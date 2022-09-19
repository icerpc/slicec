// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, Error, Warning};
use crate::grammar::Entity;
use crate::utils::attribute::IGNORE_WARNINGS;

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
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    /// The relative paths of all Slice files that have the file level `ignore_warnings` attribute.
    pub ignore_warning_file_paths: Vec<String>,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors: slice_options.warn_as_error,
            diagnostic_format: slice_options.diagnostic_format,
            ignore_warning_file_paths: Vec::new(),
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        (self.error_count != 0) || (self.treat_warnings_as_errors && (self.warning_count != 0))
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Consumes the diagnostic reporter, returning all the diagnostics that have been reported with it.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn report_error(&mut self, error: Error) {
        self.error_count += 1;
        self.diagnostics.push(Diagnostic::Error(error));
    }

    pub fn report_warning(&mut self, warning: Warning, entity: &dyn Entity) {
        self.warning_count += 1;
        if !entity.has_attribute(IGNORE_WARNINGS, true)
            && !warning
                .span
                .as_ref()
                .map_or(false, |s| self.ignore_warning_file_paths.iter().any(|f| *f == s.file))
        {
            self.diagnostics.push(Diagnostic::Warning(warning));
        }
    }
}
