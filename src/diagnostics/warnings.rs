// Copyright (c) ZeroC, Inc. All rights reserved.

use super::{DiagnosticReporter, Note};
use crate::grammar::{AttributeKind, Entity};
use crate::implement_error_functions;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Warning {
    pub(super) kind: WarningKind,
    pub(super) span: Span,
    pub(super) notes: Vec<Note>,
}

impl Warning {
    pub fn new(kind: WarningKind, span: &Span) -> Self {
        Warning {
            kind,
            span: span.to_owned(),
            notes: Vec::new(),
        }
    }

    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note::new(message, span));
        self
    }

    pub fn report(self, reporter: &mut DiagnosticReporter, entity: &dyn Entity) {
        // Returns true if the Slice file has the file level `ignoreWarnings` attribute with no arguments (ignoring all
        // warnings), or if it has an argument matching the error code of the warning.
        if match reporter.file_level_ignored_warnings.get(&self.span.file) {
            None => false,
            Some(args) if args.is_empty() => true,
            Some(args) => args.contains(&self.error_code().to_owned()),
        } {
            // Do not push the warning to the diagnostics vector
            return;
        }

        // Returns true if the entity (or its parent) has the`ignoreWarnings` attribute with no arguments (ignoring all
        // warnings), or if it has an argument matching the error code of the warning.
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
    /// The user-supplied doc comment indicated that the operation should contain a parameter that it does not have.
    ///
    /// # Fields
    ///
    /// * `identifier` - The name of the parameter from the user-supplied doc comment.
    ExtraParameterInDocComment(String),

    /// The user-supplied doc comment indicated that the operation should return a value, but the operation does not.
    ExtraReturnValueInDocComment,

    /// The user-supplied doc comment indicated that the entity should throw, but the entity does not support throwing.
    ///
    /// # Fields
    ///
    /// * `kind` - The kind of that entity that was indicated to throw.
    /// * `identifier` - The identifier of that entity that was indicated to throw.
    ExtraThrowInDocComment(String, String),

    /// The user-supplied doc comment link referenced an entity that does not exist.
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the entity that was referenced.
    InvalidDocCommentLinkIdentifier(String),

    /// The user-supplied doc comment tag is invalid.
    ///
    /// # Fields
    ///
    /// * `tag` - The doc comment tag
    InvalidDocCommentTag(String),

    /// The code references a Slice entity that is deprecated.
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the deprecated entity.
    /// * `deprecation_reason` - The reason why the slice entity was deprecated. If not supplied it will an empty
    ///   string.
    UseOfDeprecatedEntity(String, String),

    /// The user applied an attribute on a type that will result in no changes
    ///
    /// # Fields
    /// * `attribute` - The attribute that was applied
    /// * `kind` - The entity the user applied the attribute to.
    InconsequentialUseOfAttribute(String, String),
}

implement_error_functions!(
    WarningKind,
    (
        "W001",
        WarningKind::ExtraParameterInDocComment,
        format!("doc comment has a param tag for '{param_name}', but there is no parameter by that name"),
        param_name
    ),
    (
        "W002",
        WarningKind::ExtraReturnValueInDocComment,
        "void operation must not contain doc comment return tag"
    ),
    (
        "W003",
        WarningKind::ExtraThrowInDocComment,
        format!("doc comment indicates that {kind} `{identifier}` throws, however, only operations can throw"),
        kind,
        identifier
    ),
    (
        "W004",
        WarningKind::InvalidDocCommentLinkIdentifier,
        format!("doc comment references an identifier `{identifier}` that does not exist"),
        identifier
    ),
    (
        "W005",
        WarningKind::InvalidDocCommentTag,
        format!("doc comment tag `{tag}` is invalid"),
        tag
    ),
    (
        "W006",
        WarningKind::UseOfDeprecatedEntity,
        format!("`{identifier}` is deprecated {deprecation_reason}"),
        identifier,
        deprecation_reason
    ),
    (
        "W007",
        WarningKind::InconsequentialUseOfAttribute,
        format!("`{attribute}` does not have any effect on {kind}"),
        attribute,
        kind
    )
);
