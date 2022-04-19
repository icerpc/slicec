// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::{Location, SliceFile};
use std::collections::HashMap;
use std::mem;
use std::fmt;

#[derive(Default)]
pub struct ErrorReporter {
    /// Vector where all the errors are stored, in the order they're reported.
    errors: Vec<Error>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
}

impl std::fmt::Debug for ErrorReporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorReporter {{ error_count: {}, warning_count: {} }}",
               self.error_count, self.warning_count)
    }
}

impl ErrorReporter {
    /// Checks if any errors have been reported during compilation.
    /// This doesn't include notes, and only includes warnings if [`include_warnings`] is set.
    pub fn has_errors(&self, include_warnings: bool) -> bool {
        (self.error_count != 0) || (include_warnings && (self.warning_count != 0))
    }

    pub fn report(&mut self,
        message: String,
        location: Option<&Location>,
        severity: ErrorLevel)
    {
        match severity {
            ErrorLevel::Note => {}
            ErrorLevel::Warning => self.warning_count += 1,
            ErrorLevel::Error => self.error_count += 1,
            ErrorLevel::Critical => {
                //TODO:  Report the error and exit immediately.
            }
        };
        self.errors.push(Error { message, location: location.cloned(), severity })
    }

    pub fn report_note(&mut self, message: String, location: Option<&Location>) {
        self.report(message, location, ErrorLevel::Note);
    }

    pub fn report_warning(&mut self, message: String, location: Option<&Location>) {
        self.report(message, location, ErrorLevel::Warning);
    }

    pub fn report_error(&mut self, message: String, location: Option<&Location>) {
        self.report(message, location, ErrorLevel::Error);
    }

    pub fn report_critical(&mut self, message: String, location: Option<&Location>) {
        self.report(message, location, ErrorLevel::Critical);
    }


    /// Writes the errors stored in the handler to stderr, along with any locations and snippets.
    pub fn print_errors(&mut self, slice_files: &HashMap<String, SliceFile>) {
        for error in mem::take(&mut self.errors).into_iter() {
            let prefix = match error.severity {
                ErrorLevel::Note =>    "note",
                ErrorLevel::Warning => "warning",
                ErrorLevel::Error =>   "error",
                ErrorLevel::Critical => "critical",
            };

            // Insert the prefix at the start of the message.
            let mut message = prefix.to_owned() + ": " + &error.message;

            if let Some(location) = error.location {
                // Specify the location where the error starts on its own line after the message.
                message = format!(
                    "{}\n@ '{}' ({},{})",
                    message, &location.file, location.start.0, location.start.1
                );

                // If the location spans between two positions, add a snippet from the slice file.
                if location.start != location.end {
                    message += ":\n";
                    let file = slice_files
                        .get(&location.file)
                        .expect("Slice file not in file map!");
                    message += &file.get_snippet(location.start, location.end);
                } else {
                    message += "\n";
                }
            }
            // Print the message to stderr.
            eprintln!("{}", message);
        }
    }

    /// Returns the total number of errors and warnings reported through the error handler.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    // #[cfg(test)] //TODO:
    pub fn assert_errors(&self, expected_errors: &[&str]) {
        assert_eq!(self.errors.len(), expected_errors.len());
        for (i, error) in self.errors.iter().enumerate() {
            assert_eq!(error.message, expected_errors[i]);
        }
    }
}

pub struct Error {
    pub message: String,
    pub location: Option<Location>,
    pub severity: ErrorLevel,
}

pub enum ErrorLevel {
    Critical,
    Error,
    Warning,
    Note,
}
