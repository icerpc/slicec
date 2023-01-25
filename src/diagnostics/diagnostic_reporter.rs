// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::Diagnostic;
use crate::grammar::{Attributable, Attribute};
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
    // The list of warnings to ignore from the 'ignore-warnings' flag.
    // Some([]) means ignore all warnings.
    // Some([...]) means ignore the specified warnings.
    // None means don't ignore any warnings.
    pub ignored_warnings: Option<Vec<String>>,
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
            ignored_warnings: slice_options.ignore_warnings.clone(),
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

    pub(super) fn report(&mut self, diagnostic: impl Into<Diagnostic>) {
        let diagnostic = diagnostic.into();
        match &diagnostic {
            Diagnostic::Error(_) => self.error_count += 1,
            Diagnostic::Warning(_) => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }

    /// Adds an entry into this reporter's `file_level_ignored_warnings` map for the specified slice file.
    pub(crate) fn add_file_level_ignore_warnings_for(&mut self, slice_file: &SliceFile) {
        // Vector all of ignore warning attributes. The attribute can be specified multiple times. An empty inner vector
        // indicates that all warnings should be ignored.
        // eg. [ignoreWarnings]
        //     [ignoreWarnings("W001", "W002")]
        let ignore_warning_attributes = slice_file
            .attributes(false)
            .into_iter()
            .filter_map(Attribute::match_ignore_warnings)
            .collect::<Vec<Vec<_>>>();

        // If any of the vectors are empty then we just ignore all warnings.
        if ignore_warning_attributes.iter().any(Vec::is_empty) {
            self.file_level_ignored_warnings
                .insert(slice_file.relative_path.clone(), Vec::new());
        } else if !ignore_warning_attributes.is_empty() {
            // Otherwise we are ignoring the specified warnings.
            self.file_level_ignored_warnings
                .insert(slice_file.relative_path.clone(), ignore_warning_attributes.concat());
        }
        // else we are not ignoring any warnings.
    }
}
