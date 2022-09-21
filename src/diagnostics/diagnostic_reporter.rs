// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, Error, Warning};
use crate::grammar::Entity;
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
    /// The relative paths of all Slice files that have the file level `ignore_warnings` attribute.
    pub file_level_ignored_warnings: HashMap<String, Vec<String>>,
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

        // Returns true if the entity has specified that it should ignore the reported warning.
        let entity_is_ignoring_warning = entity
            .get_ignored_warnings(true)
            .map(|args| {
                if args.is_empty() {
                    // The ignore_warnings attribute has no arguments which indicates that all warnings should be
                    // ignored.
                    true
                } else if let Some(error_code) = warning.error_code() {
                    // The ignore_warnings attribute has arguments which indicates that only specific warnings should be
                    // ignored.
                    args.iter().any(|arg| arg == error_code)
                } else {
                    // The warning does not have an error code so it cannot be ignored.
                    false
                }
            })
            .unwrap_or(false);

        // Returns true if the file has specified that it should ignore the reported warning.
        let file_is_ignoring_warning = warning.span.as_ref().map_or(false, |s| {
            self.file_level_ignored_warnings.iter().any(|map| {
                !(map.0 != &s.file
                    || !map.1.is_empty() && !(warning.error_code().map_or(false, |e| map.1.contains(&e.to_owned()))))
            })
        });

        println!("file ignoring warnings {file_is_ignoring_warning}; entity ignoring {entity_is_ignoring_warning}");
        if !entity_is_ignoring_warning && !file_is_ignoring_warning {
            self.diagnostics.push(Diagnostic::Warning(warning));
        }
    }
}
