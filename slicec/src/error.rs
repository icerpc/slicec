// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::Symbol;
use crate::slice_file::{Location, SliceFile};
use std::collections::HashMap;
use std::mem;

/// The Error struct holds information describing a slice related error. It's only used internally
/// by this module. Errors should only be created when reporting them through an [`ErrorHandler`]
/// with an `into` conversion.
/// # Examples
/// ```ignore
/// handler.report_error("error!".into());              // just an error message
/// handler.report_error(("error!", location).into());  // error message with explicit location
/// handler.report_error(("error!", my_struct).into()); // error message and the symbol causing it.
/// ```
#[derive(Debug)]
pub struct Error {
    /// The message to print when the error is displayed.
    message: String,
    /// The error's location. If set, the location and a code snippet are printed with the error.
    location: Option<Location>,
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error { message, location: None }
    }
}

impl From<(String, Location)> for Error {
    fn from((message, location): (String, Location)) -> Self {
        Error { message, location: Some(location) }
    }
}

impl<T: Symbol + ?Sized> From<(String, &T)> for Error {
    fn from((message, symbol): (String, &T)) -> Self {
        Error { message, location: Some(symbol.location().clone()) }
    }
}

impl<T: Symbol + ?Sized> From<(String, &mut T)> for Error {
    fn from((message, symbol): (String, &mut T)) -> Self {
        Error { message, location: Some(symbol.location().clone()) }
    }
}

/// ErrorHolder has variants describing the possible severity of a slice error.
/// Each variant holds the error it's describing.
#[derive(Debug)]
enum ErrorHolder {
    /// High severity.
    /// Errors end compilation immediately after the current compilation phase finishes.
    Error(Error),
    /// Low severity.
    /// Warnings only impact compilation if `warn-as-error` is set, where they're treated as errors.
    Warning(Error),
    /// No severity.
    /// Notes have no impact on compilation and are purely informative.
    Note(Error),
}

/// The ErrorHandler temporarily stores errors, warnings, and notes reported by
/// the compiler during execution, and provides methods for reporting them.
///
/// Errors can reference elements and code snippets that might not be accessible in the scope
/// they're reported from (or they may not of been parsed yet). So instead of immediately reporting
/// them, they're stored by the ErrorHandler and only reported when [`print_errors`] is called.
#[derive(Debug, Default)]
pub struct ErrorHandler {
    /// Vector where all the errors are stored, in the order they're reported.
    errors: Vec<ErrorHolder>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
}

impl ErrorHandler {
    /// Checks if any errors have been reported during compilation.
    /// This doesn't include notes, and only includes warnings if [`include_warnings`] is set.
    pub fn has_errors(&self, include_warnings: bool) -> bool {
        (self.error_count != 0) || (include_warnings && (self.warning_count != 0))
    }

    /// Reports an error. If the error's location is set, a code snippet is printed beneath it.
    ///
    /// It is recommended to construct the Error with an `into` conversion, instead of directly.
    /// # Examples
    /// ```ignore
    /// handler.report_error("error!".into());              // without location
    /// handler.report_error(("error!", location).into());  // with location
    /// ```
    pub fn report_error(&mut self, error: Error) {
        self.errors.push(ErrorHolder::Error(error));
        self.error_count += 1;
    }

    /// Reports a warning. If the error's location is set, a code snippet is printed beneath it.
    ///
    /// It is recommended to construct the Error with an `into` conversion, instead of directly.
    /// # Examples
    /// ```ignore
    /// handler.report_warning("warning!".into());              // without location
    /// handler.report_warning(("warning!", location).into());  // with location
    /// ```
    pub fn report_warning(&mut self, warning: Error) {
        self.errors.push(ErrorHolder::Warning(warning));
        self.warning_count += 1;
    }

    /// Reports a note. If the error's location is set, a code snippet is printed beneath it.
    ///
    /// It is recommended to construct the Error with an `into` conversion, instead of directly.
    /// # Examples
    /// ```ignore
    /// handler.report_note("note!".into());              // without location
    /// handler.report_note(("note!", location).into());  // with location
    /// ```
    pub fn report_note(&mut self, note: Error) {
        self.errors.push(ErrorHolder::Note(note));
    }

    /// Writes the errors stored in the handler to stderr, along with any locations and snippets.
    pub fn print_errors(&mut self, slice_files: &HashMap<String, SliceFile>) {
        for error_holder in mem::take(&mut self.errors).into_iter() {
            // Unwrap the error into its fields, and get its error severity prefix.
            let (mut message, location, prefix) = match error_holder {
                ErrorHolder::Error(error) => (error.message, error.location, "error: "),
                ErrorHolder::Warning(error) => (error.message, error.location, "warning: "),
                ErrorHolder::Note(error) => (error.message, error.location, "note: "),
            };
            // Insert the prefix at the start of the message.
            message.insert_str(0, prefix);

            if let Some(loc) = location {
                // Specify the location where the error starts on its own line after the message.
                message = format!(
                    "{}\n@ '{}' ({},{}):\n",
                    message, &loc.file, loc.start.0, loc.start.1
                );

                // If the location spans between two positions, add a snippet from the slice file.
                if loc.start != loc.end {
                    let file = slice_files
                        .get(&loc.file)
                        .expect("Slice file not in file map!");
                    message += &file.get_snippet(loc.start, loc.end);
                }
            }
            // Print the message to stderr.
            eprintln!("{}\n", message);
        }
    }

    /// Returns the total number of errors and warnings reported through the error handler.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }
}
