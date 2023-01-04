// Copyright (c) ZeroC, Inc. All rights reserved.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;

mod diagnostic_reporter;
mod errors;
mod warnings;

use crate::slice_file::Span;

pub use self::diagnostic_reporter::DiagnosticReporter;
pub use self::errors::{Error, ErrorKind};
pub use self::warnings::{Warning, WarningKind};

/// A diagnostic is a message that is reported to the user during compilation. It can be an [Error] or a [Warning].
#[derive(Debug)]
pub enum Diagnostic {
    Error(Error),
    Warning(Warning),
}

impl Diagnostic {
    /// Returns the [Span] of the diagnostic if it has one.
    pub fn span(&self) -> Option<&Span> {
        match self {
            Diagnostic::Error(error) => error.span.as_ref(),
            Diagnostic::Warning(warning) => Some(&warning.span),
        }
    }

    /// Returns a slice of [Note]s associated with the diagnostic.
    pub fn notes(&self) -> &[Note] {
        match self {
            Diagnostic::Error(error) => &error.notes,
            Diagnostic::Warning(warning) => &warning.notes,
        }
    }

    /// Returns the error code of the diagnostic if it has one.
    pub fn error_code(&self) -> Option<&str> {
        match self {
            Diagnostic::Error(error) => error.error_code(),
            Diagnostic::Warning(warning) => Some(warning.error_code()),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Diagnostic::Error(error) => write!(f, "{}", error),
            Diagnostic::Warning(warning) => write!(f, "{}", warning),
        }
    }
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Diagnostic", 4)?;
        let severity = match &self {
            Diagnostic::Error(_) => "error",
            Diagnostic::Warning(_) => "warning",
        };
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("severity", severity)?;
        state.serialize_field("span", &self.span())?;
        state.serialize_field("notes", self.notes())?;
        state.serialize_field("error_code", &self.error_code())?;
        state.end()
    }
}

impl From<Error> for Diagnostic {
    fn from(error: Error) -> Self {
        Diagnostic::Error(error)
    }
}

impl From<Warning> for Diagnostic {
    fn from(warning: Warning) -> Self {
        Diagnostic::Warning(warning)
    }
}

/// Additional information about a diagnostic. For example, indicating where the encoding of a Slice1 encoded Slice file
/// was defined.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

impl Note {
    /// Creates a new [Note] with the given message and span.
    pub fn new(message: impl Into<String>, span: Option<&Span>) -> Self {
        Note {
            message: message.into(),
            span: span.cloned(),
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// A macro that implements the `error_code` and `message` functions for [WarningKind] and [ErrorKind] enums.
#[macro_export]
macro_rules! implement_diagnostic_functions {
    (WarningKind, $(($code:expr, $kind:path, $message:expr $(, $variant:ident)* )),*) => {

        impl $crate::diagnostics::Warning {
            pub fn all_codes() -> Vec<&'static str> {
                vec![$($code),*]
            }
        }

        impl WarningKind {
            pub fn error_code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error $kind, $($variant),*) => $code,
                    )*
                }
            }
        }

        impl std::fmt::Display for WarningKind {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let message: String = match self {
                    $(
                        implement_diagnostic_functions!(@description $kind, $($variant),*) => $message.into(),
                    )*
                };
                write!(f, "{}", message)
            }
        }
    };

    (ErrorKind, $(($($code:literal,)? $kind:path, $message:expr $(, $variant:ident)* )),*) => {
        impl ErrorKind {
            pub fn error_code(&self) -> Option<&str> {
                match self {
                    $(
                        implement_diagnostic_functions!(@error $kind, $($variant),*) => implement_diagnostic_functions!(@code $($code)?),
                    )*
                }
            }
        }

        impl std::fmt::Display for ErrorKind {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let message: String = match self {
                    $(
                        implement_diagnostic_functions!(@description $kind, $($variant),*) => $message.into(),
                    )*
                };
                write!(f, "{}", message)
            }
        }
    };

    (@code $code:literal) => {
        Some($code)
    };

    (@code) => {
        None
    };

    (@error $kind:path,) => {
        $kind
    };

    (@error $kind:path, $($variant:ident),+) => {
        $kind {..}
    };

    (@description $kind:path,) => {
        $kind
    };

    (@description $kind:path, $($variant:ident),+) => {
        $kind{$($variant),*}
    };
}
