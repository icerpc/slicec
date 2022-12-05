// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::Diagnostic;

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
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    /// The relative paths of all Slice files that have the file level `ignoreWarnings` attribute.
    pub file_level_ignored_warnings: HashMap<String, Vec<String>>,
    // If true, diagnostic output will not be styled.
    pub disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors: slice_options.warn_as_error,
            diagnostic_format: slice_options.diagnostic_format,
            file_level_ignored_warnings: HashMap::new(),
            disable_color: slice_options.disable_color,
        }
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

    /// Consumes the diagnostic reporter, returning all the diagnostics that have been reported with it.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn report(&mut self, diagnostic: impl Into<Diagnostic>) {
        let diagnostic = diagnostic.into();
        match diagnostic {
            Diagnostic::Error(error) => {
                self.error_count += 1;
                self.diagnostics.push(Diagnostic::Error(error));
            }
            Diagnostic::Warning(warning) => {
                self.warning_count += 1;
                self.diagnostics.push(Diagnostic::Warning(warning));
            }
        }
    }
}
