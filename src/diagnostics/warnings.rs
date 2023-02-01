// Copyright (c) ZeroC, Inc. All rights reserved.

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

    /// The user made a syntactical mistake in a doc comment.
    DocCommentSyntax {
        /// Message explaining the mistake to the user.
        message: String,
    },

    /// The user specified an unknown tag type.
    UnknownDocCommentTag {
        /// The unknown tag's keyword.
        tag: String,
    },

    /// The user didn't have a tag keyword after an '@' character.
    MissingDocCommentTag,

    /// An inline tag is missing its closing brace. Ex: `{@link Foo` (there's no closing '}').
    UnterminatedInlineTag,

    /// The user used a doc comment tag in a place where it was invalid to do so.
    /// Ex: Using '@param' (a block tag), in the context of an inline tag: `{@param foo}`.
    InvalidDocCommentTagUsage {
        /// The tag that was used.
        tag: String,
        /// Where the tag was used; `true` if it was used inline, `false` if it was used in a block.
        is_inline: bool,
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

    /// A doc comment link referenced an element that does not exist.
    DoesNotExist {
        /// The identifier that the link referenced.
        identifier: String,
    },

    /// A doc comment link referenced a type that cannot be referenced: primitive, sequence, or dictionary.
    LinkToInvalidElement {
        /// The kind of element the link references.
        kind: String,
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
    ("W002", WarningKind::DocCommentSyntax, message, message),
    (
        "W003",
        WarningKind::UnknownDocCommentTag,
        format!("doc comment tag '{tag}' is invalid"),
        tag
    ),
    ("W004", WarningKind::MissingDocCommentTag, "missing doc comment tag"),
    (
        "W005",
        WarningKind::UnterminatedInlineTag,
        "missing a closing '}' on an inline doc comment tag."
    ),
    (
        "W006",
        WarningKind::InvalidDocCommentTagUsage,
        format!(
            "doc comment tag '{tag}' cannot be used {}",
            if *is_inline { "inline" } else { "to start a block" },
        ),
        tag,
        is_inline
    ),
    (
        "W007",
        WarningKind::ExtraParameterInDocComment,
        format!("doc comment has a param tag for '{identifier}', but there is no parameter by that name"),
        identifier
    ),
    (
        "W008",
        WarningKind::ExtraReturnValueInDocComment,
        "void operation must not contain doc comment return tag"
    ),
    (
        "W009",
        WarningKind::ExtraThrowInDocComment,
        format!("doc comment indicates that {kind} '{identifier}' throws, however, only operations can throw"),
        kind,
        identifier
    ),
    (
        "W010",
        WarningKind::DoesNotExist,
        format!("no element with identifier '{identifier}' can be found from this scope"),
        identifier
    ),
    (
        "W011",
        WarningKind::LinkToInvalidElement,
        format!("elements of the type '{kind}' cannot be referenced in doc comments"),
        kind
    ),
    (
        "W012",
        WarningKind::UseOfDeprecatedEntity,
        format!("'{identifier}' is deprecated {deprecation_reason}"),
        identifier,
        deprecation_reason
    ),
    (
        "W013",
        WarningKind::InconsequentialUseOfAttribute,
        format!("'{attribute}' does not have any effect on {kind}"),
        attribute,
        kind
    )
);
