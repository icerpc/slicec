// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

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
}

implement_from_for_error_sub_kind!(WarningKind, DiagnosticKind::Warning);
implement_error_functions!(
    WarningKind,
    (
        WarningKind::ExtraParameterInDocComment,
        1000,
        format!("doc comment has a param tag for '{param_name}', but there is no parameter by that name"),
        param_name
    ),
    (
        WarningKind::ExtraReturnValueInDocComment,
        1001,
        "void operation must not contain doc comment return tag"
    ),
    (
        WarningKind::ExtraThrowInDocComment,
        1002,
        format!("doc comment indicates that {kind} `{identifier}` throws, however, only operations can throw"),
        kind,
        identifier
    ),
    (
        WarningKind::InvalidDocCommentLinkIdentifier,
        1002,
        format!("doc comment references an identifier `{identifier}` that does not exist"),
        identifier
    ),
    (
        WarningKind::InvalidDocCommentTag,
        1003,
        format!("doc comment tag `{tag}` is invalid"),
        tag
    ),
    (
        WarningKind::UseOfDeprecatedEntity,
        1004,
        format!("`{identifier}` is deprecated: {deprecation_reason}"),
        identifier,
        deprecation_reason
    )
);
