// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::{Location, SliceFile};
use std::collections::HashMap;

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

    /// Returns `Ok` if the compiler hasn't encountered any errors and should continue execution.
    /// Returns `Err` if the compiler has encountered an error and should exit gracefully.
    pub fn get_state(&self) -> Result<(), ()> {
        match self.has_errors() {
            false => Ok(()),
            true => Err(()),
        }
    }

    /// Returns the total number of errors and warnings reported through the error reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Returns a slice of the errors that have been reported.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Adds a new error to the error reporter. The warning count and error count are also incremented.
    fn report(&mut self, error: Error) {
        match error.severity {
            ErrorLevel::Note => {}
            ErrorLevel::Warning => self.warning_count += 1,
            ErrorLevel::Error => self.error_count += 1,
        };
        self.errors.push(error);
    }

    pub fn append_errors(&mut self, errors: Vec<Error>) {
        for error in errors {
            self.report(error);
        }
    }

    pub fn report_note(&mut self, message: impl Into<String>, location: Option<&Location>) {
        self.report(Error {
            message: message.into(),
            location: location.cloned(),
            severity: ErrorLevel::Note,
        });
    }

    pub fn report_warning(&mut self, message: impl Into<String>, location: Option<&Location>) {
        self.report(Error {
            message: message.into(),
            location: location.cloned(),
            severity: ErrorLevel::Warning,
        });
    }

    pub fn report_error(&mut self, message: impl Into<String>, location: Option<&Location>) {
        self.report(Error {
            message: message.into(),
            location: location.cloned(),
            severity: ErrorLevel::Error,
        });
    }

    /// Writes the errors stored to stderr, along with any locations and snippets.
    pub fn print_errors(&self, slice_files: &HashMap<String, SliceFile>) {
        for error in self.errors.iter() {
            let prefix = match error.severity {
                ErrorLevel::Note => "note",
                ErrorLevel::Warning => "warning",
                ErrorLevel::Error => "error",
            };

            // Insert the prefix at the start of the message.
            let mut message = prefix.to_owned() + ": " + &error.message;

            if let Some(location) = &error.location {
                // Specify the location where the error starts on its own line after the message.
                message = format!(
                    "{}\n@ '{}' ({},{})",
                    message, &location.file, location.start.0, location.start.1
                );

                // If the location spans between two positions, add a snippet from the slice file.
                if location.start != location.end {
                    message += ":\n";
                    let file = slice_files.get(&location.file).expect("Slice file not in file map!");
                    message += &file.get_snippet(location.start, location.end);
                } else {
                    message += "\n";
                }
            }
            // Print the message to stderr.
            eprintln!("{}", message);
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub location: Option<Location>,
    pub severity: ErrorLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorLevel {
    Error,
    Warning,
    Note,
}
