// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, Error, Warning};
use crate::grammar::{AttributeKind, Entity};

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
        (self.error_count != 0) || (self.treat_warnings_as_errors && (self.warning_count != 0))
    }

    /// Checks if any warnings have been reported during compilation.
    pub fn has_warnings(&self) -> bool {
        self.warning_count != 0
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

        // Returns true if the Slice file has the file level `ignoreWarnings` attribute with no arguments (ignoring all
        // warnings), or if it has an argument matching the error code of the warning.
        if match self.file_level_ignored_warnings.get(&warning.span.file) {
            None => false,
            Some(args) if args.is_empty() => true,
            Some(args) => args.contains(&warning.error_code().to_owned()),
        } {
            // Do not push the warning to the diagnostics vector
            return;
        }

        // Returns true if the entity (or its parent) has the`ignoreWarnings` attribute with no arguments (ignoring all
        // warnings), or if it has an argument matching the error code of the warning.
        if entity.attributes(true).iter().any(|a| match &a.kind {
            AttributeKind::IgnoreWarnings { warning_codes } => match warning_codes {
                Some(codes) => codes.is_empty() || codes.contains(&warning.error_code().to_owned()),
                None => true,
            },
            _ => false,
        }) {
            // Do not push the warning to the diagnostics vector
            return;
        }

        self.diagnostics.push(Diagnostic::Warning(warning));
    }
}
