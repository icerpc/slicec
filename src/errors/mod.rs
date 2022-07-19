// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::slice_file::Location;
use std::fmt;
use std::string::ToString;

mod rules;
mod warnings;

pub use self::rules::*;
pub use self::warnings::WarningKind;

// TODO: Rename this error in a future PR when Error is removed.
#[derive(Debug, Clone)]
pub struct TempError {
    pub error_kind: ErrorKind,
    pub error_code: u32,
    pub message: String,
}

impl TempError {
    pub fn new(error_kind: ErrorKind) -> Self {
        TempError {
            error_kind: error_kind.clone(),
            error_code: error_kind.error_code(),
            message: error_kind.message(),
        }
    }
}
impl fmt::Display for TempError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind.message())
    }
}

impl From<TempError> for Error {
    fn from(reeces_error: TempError) -> Self {
        let error_kind = reeces_error.clone().error_kind;
        Self {
            message: reeces_error.to_string(),
            location: error_kind.location(),
            severity: error_kind.severity(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Warning(WarningKind, Option<Location>),
    RuleError(RuleKind, Option<Location>),
    SyntaxError(WarningKind, Option<Location>),
    Note(String, Option<Location>),
}

impl ErrorKind {
    pub fn error_code(&self) -> u32 {
        match self {
            ErrorKind::Warning(warning_kind, _) => 1000 + warning_kind.error_code(),
            ErrorKind::RuleError(rule_kind, _) => 2000 + rule_kind.error_code(),
            ErrorKind::SyntaxError(syntax_kind, _) => 3000 + syntax_kind.error_code(),
            ErrorKind::Note(_, _) => 0,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ErrorKind::Warning(warning_kind, _) => warning_kind.get_description(),
            ErrorKind::RuleError(rule_kind, _) => rule_kind.get_description(),
            ErrorKind::SyntaxError(warning_kind, _) => warning_kind.get_description(),
            ErrorKind::Note(message, _) => message.clone(),
        }
    }

    pub fn severity(&self) -> ErrorLevel {
        match self {
            ErrorKind::Warning(_, _) => ErrorLevel::Warning,
            ErrorKind::RuleError(_, _) => ErrorLevel::Error,
            ErrorKind::SyntaxError(_, _) => ErrorLevel::Error,
            ErrorKind::Note(_, _) => ErrorLevel::Note,
        }
    }

    pub fn location(&self) -> Option<Location> {
        match self {
            ErrorKind::Warning(_, location) => location.clone(),
            ErrorKind::RuleError(_, location) => location.clone(),
            ErrorKind::SyntaxError(_, location) => location.clone(),
            ErrorKind::Note(_, location) => location.clone(),
        }
    }
}
