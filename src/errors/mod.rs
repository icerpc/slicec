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
}

impl ErrorKind {
    pub fn message(&self) -> String {
        match self {
            ErrorKind::Warning(warning_kind, _) => "warning: ".to_owned() + &warning_kind.get_description(),
            ErrorKind::RuleError(rule_kind, _) => "error: ".to_owned() + &rule_kind.get_description(),
            ErrorKind::SyntaxError(warning_kind, _) => "syntax error: ".to_owned() + &warning_kind.get_description(),
        }
    }

    pub fn severity(&self) -> ErrorLevel {
        match self {
            ErrorKind::Warning(_, _) => ErrorLevel::Warning,
            ErrorKind::RuleError(_, _) => ErrorLevel::Error,
            ErrorKind::SyntaxError(_, _) => ErrorLevel::Error,
        }
    }

    pub fn location(&self) -> Option<Location> {
        match self {
            ErrorKind::Warning(_, location) => location.clone(),
            ErrorKind::RuleError(_, location) => location.clone(),
            ErrorKind::SyntaxError(_, location) => location.clone(),
        }
    }
}
