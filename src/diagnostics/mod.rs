// Copyright (c) ZeroC, Inc.

use crate::slice_file::Span;
use serde::Serialize;

mod diagnostic;
mod errors;
mod lints;

pub use diagnostic::*;
pub use errors::Error;
pub use lints::Lint;

/// Stores additional information about a diagnostic.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

/// A macro that implements the `code` and `message` functions for [Lint] and [Error] enums.
#[macro_export]
macro_rules! implement_diagnostic_functions {
    (Lint, $(($kind:ident, $message:expr $(, $variant:ident)* )),*) => {
        impl Lint {
            // TODO maybe we should move this somewhere other than `Lint`? Like in `Attribute` maybe?
            /// This array contains all the valid arguments for the 'allow' attribute.
            pub const ALLOWABLE_LINT_IDENTIFIERS: [&'static str; 6] = [
                "All",
                $(stringify!($kind)),*
            ];

            pub fn code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error Lint::$kind, $($variant),*) => stringify!($kind),
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_diagnostic_functions!(@description Lint::$kind, $($variant),*) => $message.into(),
                    )*
                }
            }
        }
    };

    (Error, $(($code:literal, $kind:ident, $message:expr $(, $variant:ident)* )),*) => {
        impl Error {
            pub fn code(&self) -> &str {
                match self {
                    $(
                        implement_diagnostic_functions!(@error Error::$kind, $($variant),*) => $code,
                    )*
                }
            }

            pub fn message(&self) -> String {
                match self {
                    $(
                        implement_diagnostic_functions!(@description Error::$kind, $($variant),*) => $message.into(),
                    )*
                }
            }
        }
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
