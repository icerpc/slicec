// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::slice_file::Location;
use std::string::ToString;

mod rules;
mod warnings;

pub use self::rules::*;
pub use self::warnings::WarningKind;

pub struct ReecesError {
    pub error_kind: ErrorKind,
}

impl From<ReecesError> for Error {
    fn from(reeces_error: ReecesError) -> Self {
        let error_kind = reeces_error.error_kind;
        Self {
            message: error_kind.message(),
            location: error_kind.location(),
            severity: error_kind.severity(),
        }
    }
}

pub enum ErrorKind {
    Warning(WarningKind, Option<Location>),
    RuleError(RuleKind, Option<Location>),
    SyntaxError(WarningKind, Option<Location>),
}

impl ErrorKind {
    pub fn message(&self) -> String {
        match self {
            ErrorKind::Warning(warning_kind, _) => warning_kind.get_description(),
            ErrorKind::RuleError(rule_kind, _) => rule_kind.get_description(),
            ErrorKind::SyntaxError(warning_kind, _) => warning_kind.get_description(),
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
