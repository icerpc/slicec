// Copyright (c) ZeroC, Inc. All rights reserved.

use std::collections::HashMap;

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, DiagnosticKind};
use crate::grammar::Entity;
use crate::slice_file::SliceFile;

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

    /// Removes globally ignored warnings from the diagnostics vector.

    pub fn remove_file_level_ignored_warnings(&mut self, files: &HashMap<String, SliceFile>) {
        let ignore_warnings_files = files
            .iter()
            .filter(|(_, file)| file.attributes.iter().any(|a| a.directive == "ignore_warnings"))
            .map(|(_, file)| file.relative_path.as_str())
            .collect::<Vec<&str>>();
        self.diagnostics.retain(|d| {
            d.span
                .as_ref()
                .map_or(true, |s| !ignore_warnings_files.iter().any(|f| *f == s.file))
        });
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

    pub fn report(&mut self, diagnostic: Diagnostic) {
        match &diagnostic.diagnostic_kind {
            DiagnosticKind::Warning(_) => self.warning_count += 1,
            DiagnosticKind::LogicError(_) | DiagnosticKind::SyntaxError(_) | DiagnosticKind::IOError(_) => {
                self.error_count += 1
            }
        };
        self.diagnostics.push(diagnostic);
    }

    pub fn report_warning(&mut self, diagnostic: Diagnostic, attributable: &dyn Entity) {
        if attributable.has_attribute("ignore_warnings", true)
            || diagnostic
                .span
                .as_ref()
                .map_or(false, |s| self.ignore_warning_file_paths.iter().any(|f| *f == s.file))
        {
            return;
        }
        self.report(diagnostic);
    }
}
