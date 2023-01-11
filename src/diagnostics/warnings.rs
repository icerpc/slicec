// Copyright (c) ZeroC, Inc. All rights reserved.

use super::{DiagnosticReporter, Note};
use crate::grammar::{AttributeKind, Entity};
use crate::implement_diagnostic_functions;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Warning {
    pub(super) kind: WarningKind,
    pub(super) span: Option<Span>,
    pub(super) notes: Vec<Note>,
}

impl Warning {
    pub fn new(kind: WarningKind) -> Self {
        Warning {
            kind,
            span: None,
            notes: Vec::new(),
        }
    }

    pub fn set_span(mut self, span: &Span) -> Self {
        self.span = Some(span.to_owned());
        self
    }

    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note::new(message, span));
        self
    }

    pub fn report(self, reporter: &mut DiagnosticReporter, entity: Option<&dyn Entity>) {
        // Returns true if the Slice file has the file level `ignoreWarnings` attribute with no arguments (ignoring all
        // warnings), or if it has an argument matching the error code of the warning.
        if let Some(span) = &self.span {
            if match reporter.file_level_ignored_warnings.get(&span.file) {
                None => false,
                Some(args) if args.is_empty() => true,
                Some(args) => args.contains(&self.error_code().to_owned()),
            } {
                // Do not push the warning to the diagnostics vector
                return;
            }
        }

        if let Some(entity) = entity {
            // Returns true if the entity (or its parent) has the`ignoreWarnings` attribute with no arguments (ignoring
            // all warnings), or if it has an argument matching the error code of the warning.
            if entity.attributes(true).iter().any(|a| match &a.kind {
                AttributeKind::IgnoreWarnings { warning_codes } => match warning_codes {
                    Some(codes) => codes.is_empty() || codes.contains(&self.error_code().to_owned()),
                    None => true,
                },
                _ => false,
            }) {
                // Do not push the warning to the diagnostics vector
                return;
            }
        }

        // Do not report warnings if the user has specified the `ignore-warnings` flag.
        match reporter.ignored_warnings {
            Some(ref args) if args.is_empty() => return,
            Some(ref args) if args.contains(&self.error_code().to_owned()) => return,
            _ => {}
        }

        reporter.report(self);
    }

    pub fn error_code(&self) -> &str {
        self.kind.error_code()
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind.message())
    }
}

#[derive(Debug)]
pub enum WarningKind {
    /// The user supplied either a reference or source file more than once.
    DuplicateFile {
        /// The path of the file that was supplied more than once.
        path: String,
    },

    /// The user-supplied doc comment indicated that the operation should contain a parameter that it does not have.
    ExtraParameterInDocComment {
        /// The name of the parameter from the user-supplied doc comment.
        identifier: String,
    },

    /// The user-supplied doc comment indicated that the operation should return a value, but the operation does not.
    ExtraReturnValueInDocComment,

    /// The user-supplied doc comment indicated that the entity should throw, but the entity does not support throwing.
    ExtraThrowInDocComment {
        /// The kind of the entity that was indicated to throw.
        kind: String,
        /// The identifier of the entity that was indicated to throw.
        identifier: String,
    },

    /// The user-supplied doc comment link referenced an entity that does not exist.
    InvalidDocCommentLinkIdentifier {
        /// The identifier of the entity that was referenced.
        identifier: String,
    },

    /// The user-supplied doc comment tag is invalid.
    InvalidDocCommentTag {
        /// The doc comment tag.
        tag: String,
    },

    /// The code references a Slice entity that is deprecated.
    UseOfDeprecatedEntity {
        /// The identifier of the deprecated entity.
        identifier: String,
        /// The reason why the slice entity was deprecated. If not supplied, it defaults to an empty string.
        deprecation_reason: String,
    },

    /// The user applied an attribute on a type that will result in no changes.
    InconsequentialUseOfAttribute {
        /// The attribute that was applied.
        attribute: String,
        /// The entity the user applied the attribute to.
        kind: String,
    },
}

implement_diagnostic_functions!(
    WarningKind,
    (
        "W001",
        WarningKind::DuplicateFile,
        format!("slice file was provided more than once: '{path}'"),
        path
    ),
    (
        "W002",
        WarningKind::ExtraParameterInDocComment,
        format!("doc comment has a param tag for '{identifier}', but there is no parameter by that name"),
        identifier
    ),
    (
        "W003",
        WarningKind::ExtraReturnValueInDocComment,
        "void operation must not contain doc comment return tag"
    ),
    (
        "W004",
        WarningKind::ExtraThrowInDocComment,
        format!("doc comment indicates that {kind} '{identifier}' throws, however, only operations can throw"),
        kind,
        identifier
    ),
    (
        "W005",
        WarningKind::InvalidDocCommentLinkIdentifier,
        format!("doc comment references an identifier '{identifier}' that does not exist"),
        identifier
    ),
    (
        "W006",
        WarningKind::InvalidDocCommentTag,
        format!("doc comment tag '{tag}' is invalid"),
        tag
    ),
    (
        "W007",
        WarningKind::UseOfDeprecatedEntity,
        format!("'{identifier}' is deprecated {deprecation_reason}"),
        identifier,
        deprecation_reason
    ),
    (
        "W008",
        WarningKind::InconsequentialUseOfAttribute,
        format!("'{attribute}' does not have any effect on {kind}"),
        attribute,
        kind
    )
);
