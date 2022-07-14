// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::slice_file::Location;
use std::string::ToString;

mod rules;
mod warnings;

pub use self::rules::*;
pub use self::warnings::WarningKind;

pub enum ErrorType {
    Warning(WarningKind, Option<Location>),
    RuleError(RuleKind, Option<Location>),
    SyntaxError(WarningKind, Option<Location>),
}

impl From<ErrorType> for Error {
    fn from(error_type: ErrorType) -> Self {
        let error_kind = error_type.kind();
        Self {
            message: error_kind.to_string(),
            location: error_type.location(),
            severity: error_type.severity(),
        }
    }
}

impl ErrorType {
    pub fn severity(&self) -> ErrorLevel {
        match self {
            ErrorType::Warning(_, _) => ErrorLevel::Warning,
            ErrorType::RuleError(_, _) => ErrorLevel::Error,
            ErrorType::SyntaxError(_, _) => ErrorLevel::Error,
        }
    }

    fn kind(&self) -> &dyn ErrorKind {
        match self {
            ErrorType::Warning(kind, _) => kind,
            ErrorType::RuleError(kind, _) => kind,
            ErrorType::SyntaxError(kind, _) => kind,
        }
    }

    pub fn location(&self) -> Option<Location> {
        match self {
            ErrorType::Warning(_, location) => location.clone(),
            ErrorType::RuleError(_, location) => location.clone(),
            ErrorType::SyntaxError(_, location) => location.clone(),
        }
    }
}

pub(crate) trait ErrorKind {
    fn get_error_code(&self) -> u32;
    fn get_description(&self) -> String;
}

impl ToString for &dyn ErrorKind {
    fn to_string(&self) -> String {
        let prefix = "Warning: ";
        let description: String = self.get_description();
        let error_code = format!(" [Error code {}]", self.get_error_code());
        prefix.to_owned() + &description + error_code.as_str()
    }
}
