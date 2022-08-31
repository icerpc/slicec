// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::command_line::OutputFormat;
use crate::diagnostics::{Diagnostic, DiagnosticKind};
use crate::slice_file::Span;
use crate::SliceOptions;

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    diagnostics: Vec<Diagnostic>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
    /// If true, compilation will fail on warnings in addition to errors.
    warn_as_error: bool,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    output_format: OutputFormat,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            warn_as_error: slice_options.warn_as_error,
            output_format: slice_options.output_format,
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        (self.error_count != 0) || (self.warn_as_error && (self.warning_count != 0))
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Consumes the diagnostic reporter, returning all the diagnostics that have been reported with it.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn report(&mut self, diagnostic_kind: impl Into<DiagnosticKind>, span: Option<&Span>) {
        let diagnostic_kind: DiagnosticKind = diagnostic_kind.into();
        match diagnostic_kind {
            DiagnosticKind::Note(_) => {}
            DiagnosticKind::Warning(_) => self.warning_count += 1,
            DiagnosticKind::LogicError(_) | DiagnosticKind::SyntaxError(_) | DiagnosticKind::IOError(_) => {
                self.error_count += 1
            }
        };
        self.diagnostics.push(Diagnostic {
            diagnostic_kind,
            span: span.cloned(),
        });
    }
}
