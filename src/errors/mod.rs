// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Location;
use std::fmt;

mod error_reporter;
mod rules;
mod warnings;

pub use self::error_reporter::ErrorReporter;
pub use self::rules::RuleKind;
pub use self::warnings::WarningKind;

#[derive(Debug)]
pub struct Error {
    pub error_kind: ErrorKind,
    pub location: Option<Location>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_kind)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Rule(RuleKind),
    Warning(WarningKind),
    Note(String),
    Parse(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Rule(rule_kind) => write!(f, "{}", rule_kind.message()),
            ErrorKind::Warning(warning_kind) => write!(f, "{}", warning_kind.message()),
            ErrorKind::Note(note) => write!(f, "{}", note),
            ErrorKind::Parse(error) => write!(f, "{}", error),
        }
    }
}

impl ErrorKind {
    pub fn new(message: impl Into<String>) -> ErrorKind {
        ErrorKind::Note(message.into())
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
                        implement_kind_for_enumerator!(@description $kind, $($variant),*) => $message.into(),
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
