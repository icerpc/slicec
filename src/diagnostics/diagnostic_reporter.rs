// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{Diagnostic, DiagnosticKind};
use crate::slice_file::Span;

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
}

impl DiagnosticReporter {
    pub fn new(treat_warnings_as_errors: bool) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors,
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_diagnostics(&self) -> bool {
        (self.error_count != 0) || (self.treat_warnings_as_errors && (self.warning_count != 0))
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Consumes the diagnostic reporter, returning all the errors that have been reported with it.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn report(&mut self, error_kind: impl Into<DiagnosticKind>, span: Option<&Span>) {
        let error_kind: DiagnosticKind = error_kind.into();
        match error_kind {
            DiagnosticKind::Note(_) => {}
            DiagnosticKind::Warning(_) => self.warning_count += 1,
            DiagnosticKind::LogicError(_) | DiagnosticKind::SyntaxError(_) | DiagnosticKind::IOError(_) => {
                self.error_count += 1
            }
        };
        self.diagnostics.push(Diagnostic {
            diagnostic_kind: error_kind,
            span: span.cloned(),
        });
    }
}
