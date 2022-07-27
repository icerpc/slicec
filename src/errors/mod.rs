// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::slice_file::Location;
use std::fmt;

mod rules;
mod warnings;

pub use self::rules::*;
pub use self::warnings::WarningKind;

// TODO: Rename this error in a future PR when Error is removed.
pub struct TempError<'a> {
    pub error_kind: ErrorKind,
    pub location: Option<&'a Location>,
}

impl fmt::Display for TempError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind.as_string())
    }
}

impl From<TempError<'_>> for Error {
    fn from(temp_error: TempError) -> Self {
        Self {
            message: temp_error.to_string(),
            location: temp_error.location.cloned(),
            severity: temp_error.error_kind.severity(),
        }
    }
}

pub enum ErrorKind {
    Warning(WarningKind),
    Rule(RuleKind),
    Note(String),
}

impl ErrorKind {
    pub fn create_note(note: impl Into<String>) -> ErrorKind {
        let message = note.into();
        ErrorKind::Note(message)
    }
}

impl ErrorKind {
    pub fn as_string(&self) -> String {
        match self {
            ErrorKind::Warning(warning_kind) => warning_kind.message(),
            ErrorKind::Rule(rule_kind) => rule_kind.message(),
            ErrorKind::Note(message) => message.to_owned(),
        }
    }

    pub fn severity(&self) -> ErrorLevel {
        match self {
            ErrorKind::Warning(_) => ErrorLevel::Warning,
            ErrorKind::Rule(_) => ErrorLevel::Error,
            ErrorKind::Note(_) => ErrorLevel::Note,
        }
    }
}

#[macro_export]
macro_rules! implement_from_for_error_sub_kind {
    ($type:ty, $enumerator:path) => {
        impl From<$type> for ErrorKind {
            fn from(original: $type) -> ErrorKind {
                $enumerator(original)
            }
        }
    };
}

#[macro_export]
macro_rules! implement_kind_for_enumerator {
    ($enumerator:ty, $(($kind:path, $code:expr, $message:expr $(, $variant:pat)* )),*) => {
        impl $enumerator {
            pub fn as_error_code(&self) -> u32 {
                match self {
                    $(
                        implement_kind_for_enumerator!(@error $kind, $($variant),*) => $code,
                    )*
                }
            }
            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_kind_for_enumerator!(@description $kind, $($variant),*) => $message,
                    )*
                }
            }
        }
    };

    (@error $kind:path,) => {
        $kind
    };

    (@error $kind:path, $($variant:pat),+) => {
        $kind(..)
    };

    (@description $kind:path,) => {
        $kind
    };

    (@description $kind:path, $($variant:pat),+) => {
        $kind($($variant),*)
    };
}
