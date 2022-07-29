// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::{Error, ErrorKind};
use crate::slice_file::Location;

#[derive(Debug)]
pub struct ErrorReporter {
    /// Vector where all the errors are stored, in the order they're reported.
    errors: Vec<Error>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
    /// If true, compilation will fail on warnings in addition to errors.
    treat_warnings_as_errors: bool,
}

impl ErrorReporter {
    pub fn new(treat_warnings_as_errors: bool) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors,
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        (self.error_count != 0) || (self.treat_warnings_as_errors && (self.warning_count != 0))
    }

    /// Returns the total number of errors and warnings reported through the error reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Consumes the error reporter, returning all the errors that have been reported with it.
    pub fn into_errors(self) -> Vec<Error> {
        self.errors
    }

    pub fn report(&mut self, error_kind: impl Into<ErrorKind>, location: Option<&Location>) {
        let error_kind: ErrorKind = error_kind.into();
        match error_kind {
            ErrorKind::Note(_) => {}
            ErrorKind::Warning(_) => self.warning_count += 1,
            ErrorKind::Logic(_) | ErrorKind::Syntax(_) | ErrorKind::IO(_) => self.error_count += 1,
        };
        self.errors.push(Error {
            error_kind,
            location: location.cloned(),
        });
    }
}
