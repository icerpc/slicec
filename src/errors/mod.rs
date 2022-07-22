// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::slice_file::Location;
use std::fmt;

mod note;
mod rules;
mod warnings;

pub use self::note::Note;
pub use self::rules::*;
pub use self::warnings::WarningKind;

// TODO: Rename this error in a future PR when Error is removed.
#[derive(Debug, Clone)]
pub struct TempError<'a> {
    pub error_kind: &'a dyn ErrorType,
    pub error_code: u32,
    pub message: String,
    pub location: Option<&'a Location>,
}

impl<'a> TempError<'a> {
    pub fn new(error_kind: &'a dyn ErrorType, location: Option<&'a Location>) -> Self {
        TempError {
            error_kind,
            error_code: error_kind.error_code(),
            message: error_kind.message(),
            location,
        }
    }
}
impl fmt::Display for TempError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind.message())
    }
}

impl From<TempError<'_>> for Error {
    fn from(temp_error: TempError) -> Self {
        let error_kind = temp_error.clone().error_kind;
        Self {
            message: temp_error.to_string(),
            location: temp_error.location.cloned(),
            severity: error_kind.severity(),
        }
    }
}

pub trait ErrorType: fmt::Debug {
    fn error_code(&self) -> u32;
    fn message(&self) -> String;
    fn severity(&self) -> ErrorLevel;
}
