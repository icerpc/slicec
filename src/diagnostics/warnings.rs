// Copyright (c) ZeroC, Inc.

use crate::implement_diagnostic_functions;

#[derive(Debug)]
pub enum Warning {
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
    CouldNotResolveLink {
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

    /// The doc comment indicated that the operation should throw an invalid type.
    InvalidThrowInDocComment {
        /// The identifier of the type that was indicated to throw.
        identifier: String,
    },

    /// The operation is marked with the throws doc comment tag, but the operation does not throw anything.
    OperationDoesNotThrow {
        /// The identifier of the operation.
        identifier: String,
    },
}

implement_diagnostic_functions!(
    Warning,
    (
        "W001",
        DuplicateFile,
        format!("slice file was provided more than once: '{path}'"),
        path
    ),
    ("W002", DocCommentSyntax, message, message),
    (
        "W003",
        ExtraParameterInDocComment,
        format!("doc comment has a param tag for '{identifier}', but there is no parameter by that name"),
        identifier
    ),
    (
        "W004",
        ExtraReturnValueInDocComment,
        "void operation must not contain doc comment return tag"
    ),
    (
        "W005",
        ExtraThrowInDocComment,
        format!("doc comment indicates that {kind} '{identifier}' throws, however, only operations can throw"),
        kind,
        identifier
    ),
    (
        "W006",
        CouldNotResolveLink,
        format!("no element with identifier '{identifier}' can be found from this scope"),
        identifier
    ),
    (
        "W007",
        LinkToInvalidElement,
        format!("elements of the type '{kind}' cannot be referenced in doc comments"),
        kind
    ),
    (
        "W008",
        UseOfDeprecatedEntity,
        format!("'{identifier}' is deprecated {deprecation_reason}"),
        identifier,
        deprecation_reason
    ),
    (
        "W009",
        InconsequentialUseOfAttribute,
        format!("'{attribute}' does not have any effect on {kind}"),
        attribute,
        kind
    ),
    (
        "W010",
        InvalidThrowInDocComment,
        format!("'{identifier}' is not a throwable type"),
        identifier
    ),
    (
        "W011",
        OperationDoesNotThrow,
        format!("operation '{identifier}' does not throw anything"),
        identifier
    )
);
