// Copyright (c) ZeroC, Inc.

use super::DiagnosticLevel;
use crate::implement_diagnostic_functions;

#[derive(Debug)]
pub enum Lint {
    /// An input filename was provided multiple times.
    /// Note: it's valid to specify the same path as a source and reference file (ex: `slicec foo.slice -R foo.slice`).
    /// This is only triggered by specifying it multiple times in the same context: (ex: `slicec foo.slice foo.slice`).
    DuplicateFile {
        /// The path of the file that supplied more than once.
        path: String,
    },

    /// A deprecated Slice element was used.
    Deprecated {
        /// The element's identifier.
        identifier: String,

        /// The reason the element was deprecated (if specified).
        reason: Option<String>,
    },

    /// A syntactical mistake in a doc-comment.
    MalformedDocComment { message: String },

    /// A doc comment contains an incorrect tag. Either:
    /// - The tag itself is incorrect. Ex: using `@throws` on an element that can't or doesn't throw an exception.
    /// - The tag describes something incorrect. Ex: specifying `@param foo` when no parameter named "foo" exists.
    IncorrectDocComment { message: String },

    /// A link in a doc-comment couldn't be resolved. Either:
    /// - The link pointed to an un-linkable element, e.g. a module, primitive, sequence, or dictionary.
    /// - The link pointed to a non-existent element.
    BrokenDocLink { message: String },
}

impl Lint {
    /// Returns the default diagnostic level this lint should use when reporting violations.
    pub fn get_default_level(&self) -> DiagnosticLevel {
        match self {
            Self::DuplicateFile { .. } => DiagnosticLevel::Warning,
            Self::Deprecated { .. } => DiagnosticLevel::Warning,
            Self::MalformedDocComment { .. } => DiagnosticLevel::Warning,
            Self::BrokenDocLink { .. } => DiagnosticLevel::Warning,
            Self::IncorrectDocComment { .. } => DiagnosticLevel::Warning,
        }
    }
}

implement_diagnostic_functions!(
    Lint,
    (
        DuplicateFile,
        format!("slice file was provided more than once: '{path}'"),
        path
    ),
    (
        Deprecated,
        if let Some(reason) = reason {
            format!("'{identifier}' is deprecated: {reason}")
        } else {
            format!("'{identifier}' is deprecated")
        },
        identifier,
        reason
    ),
    (MalformedDocComment, message, message),
    (IncorrectDocComment, message, message),
    (BrokenDocLink, message, message)
);
