// Copyright (c) ZeroC, Inc.

use super::{DiagnosticReporter, Note};
use crate::implement_diagnostic_functions;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Warning {
    pub(super) kind: WarningKind,
    pub(super) span: Option<Span>,
    pub(super) scope: Option<String>,
    pub(super) notes: Vec<Note>,
}

impl Warning {
    pub fn new(kind: WarningKind) -> Self {
        Warning {
            kind,
            span: None,
            scope: None,
            notes: Vec::new(),
        }
    }

    pub fn set_span(mut self, span: &Span) -> Self {
        self.span = Some(span.to_owned());
        self
    }

    pub fn set_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note::new(message, span));
        self
    }

    pub fn report(self, diagnostic_reporter: &mut DiagnosticReporter) {
        diagnostic_reporter.report(self);
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
