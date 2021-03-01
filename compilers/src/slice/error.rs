
use crate::util::{Location, SliceFile};
use std::collections::HashMap;

//------------------------------------------------------------------------------
// SliceError
//------------------------------------------------------------------------------
/// The SliceError struct holds information describing a slice error. It's only used internally by this module.
/// Errors should only be created when reporting them through an [`ErrorHandler`] with an `into` conversion.
#[derive(Clone, Debug)]
pub struct SliceError {
    /// The message to print when the error is displayed.
    message: String,
    /// The location where the error occurred. If set, the location and a code snippet will be printed with the error.
    location: Option<Location>,
}

impl From<&str> for SliceError {
    fn from(message: &str) -> Self {
        SliceError {
            message: message.to_owned(),
            location: None
        }
    }
}

impl From<(&str, Location)> for SliceError {
    fn from((message, location): (&str, Location)) -> Self {
        SliceError {
            message: message.to_owned(),
            location: Some(location)
        }
    }
}

//------------------------------------------------------------------------------
// ErrorHolder
//------------------------------------------------------------------------------
/// ErrorHolder has variants describing the possible severity of a slice error.
/// Each variant holds the error it's describing.
#[derive(Clone, Debug)]
enum ErrorHolder {
    /// High severity. Errors will cause compilation to end prematurely after the current compilation phase is finished.
    Error(SliceError),
    /// Low severity. Warnings only impact compilation if `warn-as-error` is set, as then they're treated like errors.
    Warning(SliceError),
    /// No severity. Notes have no impact on compilation and are purely informative.
    Note(SliceError),
}

//------------------------------------------------------------------------------
// ErrorHandler
//------------------------------------------------------------------------------
/// The ErrorHandler stores all errors, warnings, and notes reported by the compiler during execution,
/// and provides methods for reporting them.
///
/// Errors can reference definitions and code snippets that might not be accessible in the scope they're reported from
/// (or they may not of been parsed yet). So instead of immediately reporting them, they're stored by the ErrorHandler
/// and only reported when [`print_errors`] is called near the end of the compiler's execution.
#[derive(Debug, Default)]
pub struct ErrorHandler {
    /// Vector where all the errors are stored, in the order they're reported in.
    errors: Vec<ErrorHolder>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
}

impl ErrorHandler {
    /// Checks if any errors have been reported yet.
    /// This doesn't include notes, and only includes warnings if [`include_warnings`] is set.
    pub fn has_errors(&self, include_warnings: bool) -> bool {
        (self.error_count != 0) || (include_warnings && (self.warning_count != 0))
    }

    /// Reports an error. If the error's location is set, a code snippet will be printed beneath it.
    ///
    /// It is recommended to construct the SliceError with an `into` conversion, instead of directly.
    ///
    /// # Examples
    /// ```
    /// handler.report_error("error!".into());              // without location
    /// handler.report_error(("error!", location).into());  // with location
    /// ```
    pub fn report_error(&mut self, error: SliceError) {
        self.errors.push(ErrorHolder::Error(error));
        self.error_count += 1;
    }

    /// Reports a warning. If the error's location is set, a code snippet will be printed beneath it.
    ///
    /// It is recommended to construct the SliceError with an `into` conversion, instead of directly.
    ///
    /// # Examples
    /// ```
    /// handler.report_warning("warning!".into());              // without location
    /// handler.report_warning(("warning!", location).into());  // with location
    /// ```
    pub fn report_warning(&mut self, warning: SliceError) {
        self.errors.push(ErrorHolder::Warning(warning));
        self.warning_count += 1;
    }

    /// Reports a note. If the error's location is set, a code snippet will be printed beneath it.
    ///
    /// It is recommended to construct the SliceError with an `into` conversion, instead of directly.
    ///
    /// # Examples
    /// ```
    /// handler.report_note("note!".into());              // without location
    /// handler.report_note(("note!", location).into());  // with location
    /// ```
    pub fn report_note(&mut self, note: SliceError) {
        self.errors.push(ErrorHolder::Note(note));
    }

    /// Writes any errors, warnings, or notes stored in the handler to stderr,
    /// along with their relevant locations and code snippets.
    ///
    /// This method consumes the ErrorHandler to ensure it can only be called once (shortly before the program exits).
    pub fn print_errors(self, slice_files: &HashMap<String, SliceFile>){
        for error_holder in self.errors.into_iter() {
            // Unwrap the error into it's fields, and get the prefix corresponding to the error severity.
            let (mut message, location, prefix) = match error_holder {
                ErrorHolder::Error(error)   => { (error.message, error.location, "error: ") },
                ErrorHolder::Warning(error) => { (error.message, error.location, "warning: ")},
                ErrorHolder::Note(error)    => { (error.message, error.location, "note: ")},
            };
            // Insert the prefix at the start of the message.
            message.insert_str(0, prefix);

            // Check if the error included a location
            if let Some(value) = location {
                // Add the string `@filename: (line,col)` on the next line in the message.
                message = format!("{}\n@{}: {:?}", message, &value.file, value.start);

                // If the location spans two file positions, add a snippet from the slice file into the error message.
                if value.start != value.end {
                    let file = match slice_files.get(&value.file) {
                        Some(file) => file,
                        None => panic!("No slice file named '{}' is in the file map!", &value.file),
                    };
                    message += file.get_snippet(value.start, value.end);
                }
            }
            // Print the message to stderr.
            eprintln!("{}\n", message);
        }

        // Print the total number of errors and warnings.
        println!("\n\terrors:{}\n\twarnings:{}\n", self.error_count, self.warning_count);
    }
}
