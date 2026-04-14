// Copyright (c) ZeroC, Inc.

use super::DiagnosticLevel;

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
    /// - The tag itself is incorrect. Ex: using `@returns` on an operation doesn't return anything.
    /// - The tag describes something incorrect. Ex: specifying `@param foo` when no parameter named "foo" exists.
    IncorrectDocComment { message: String },

    /// A link in a doc-comment couldn't be resolved. Either:
    /// - The link pointed to an un-linkable element, e.g. a module, result, sequence, dictionary, or primitive.
    /// - The link pointed to a non-existent element.
    BrokenDocLink { message: String },
}

impl Lint {
    /// Returns the default [`DiagnosticLevel`] this lint violation should be reported with unless affected by
    /// attributes (like '[allow(...)]'), or command-line options (like '--allow').
    pub fn default_diagnostic_level(&self) -> DiagnosticLevel {
        match self {
            Self::DuplicateFile { .. } => DiagnosticLevel::Warning,
            Self::Deprecated { .. } => DiagnosticLevel::Warning,
            Self::MalformedDocComment { .. } => DiagnosticLevel::Warning,
            Self::BrokenDocLink { .. } => DiagnosticLevel::Warning,
            Self::IncorrectDocComment { .. } => DiagnosticLevel::Warning,
        }
    }

    /// Returns the name of the lint which was violated.
    pub fn lint_name(&self) -> &'static str {
        match self {
            Self::DuplicateFile { .. } => "DuplicateFile",
            Self::Deprecated { .. } => "Deprecated",
            Self::MalformedDocComment { .. } => "MalformedDocComment",
            Self::IncorrectDocComment { .. } => "IncorrectDocComment",
            Self::BrokenDocLink { .. } => "BrokenDocLink",
        }
    }

    /// Returns a message describing this lint violation in detail.
    pub fn message(&self) -> String {
        match self {
            Self::DuplicateFile { path } => format!("slice file was provided more than once: '{path}'"),

            Self::Deprecated { identifier, reason } => {
                if let Some(reason) = reason {
                    format!("'{identifier}' is deprecated: {reason}")
                } else {
                    format!("'{identifier}' is deprecated")
                }
            }

            Self::MalformedDocComment { message } => format!("malformed doc comment: {message}"),

            Self::IncorrectDocComment { message } => format!("incorrect doc comment: {message}"),

            Self::BrokenDocLink { message } => format!("broken doc link: {message}"),
        }
    }
}
