// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::DiagnosticKind;
use crate::grammar::Encoding;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum LogicErrorKind {
    // ----------------  Attribute Errors ---------------- //
    /// Used to indicate when the compress attribute cannot be applied
    CompressAttributeCannotBeApplied,

    /// Used to indicate when the deprecated attribute cannot be applied
    ///
    /// # Fields
    ///
    /// * `kind` - The kind which the deprecated attribute was applied to
    DeprecatedAttributeCannotBeApplied(String),

    // ----------------  Argument Errors ---------------- //
    /// The provided argument is not supported for the given method
    ///
    /// # Fields
    ///
    /// * `argument_name` - The name of the argument
    /// * `method_name` - The name of the method
    ArgumentNotSupported(String, String),

    // ---------------- Dictionary Errors ---------------- //
    /// Dictionaries cannot use optional types as keys
    KeyMustBeNonOptional,

    /// An unsupported type was used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the type that was used as a dictionary key type
    KeyTypeNotSupported(String),

    /// Struct contains a member that cannot be used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `struct_identifier` - The identifier of the struct
    StructKeyContainsDisallowedType(String),

    /// Structs must be compact to be used as a dictionary key type
    StructKeyMustBeCompact,

    // ----------------  Encoding Errors ---------------- //
    /// The provided kind with identifier is not supported in the specified encoding
    ///
    /// # Fields
    ///
    /// * `kind` - The kind that was is not supported
    /// * `identifier` - The identifier of the kind that is not supported
    /// * `encoding` - The encoding that was specified
    NotSupportedWithEncoding(String, String, Encoding),

    /// Optional are not supported in the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    OptionalsNotSupported(Encoding),

    /// Streamed parameters are not supported with the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    StreamedParametersNotSupported(Encoding),

    /// An unsupported type was used in the specified encoding
    ///
    /// # Fields
    ///
    /// * `kind` - The name of the kind that was used in the specified encoding
    /// * `encoding` - The encoding that was specified
    UnsupportedType(String, Encoding),

    // ----------------  Enum Errors ---------------- //
    /// Enumerators must be unique
    ///
    /// # Fields
    ///
    /// * `enumerator_identifier` - The identifier of the enumerator
    CannotHaveDuplicateEnumerators(String),

    /// Enums cannot have optional underlying types
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum
    CannotUseOptionalUnderlyingType(String),

    /// An enumerator was found that was out of bounds of the underlying type of the parent enum
    ///
    /// # Fields
    ///
    /// * `enumerator_identifier` - The identifier of the enumerator
    /// * `value` - The value of the out of bounds enumerator
    /// * `min` - The minimum value of the underlying type of the enum
    /// * `max` - The maximum value of the underlying type of the enum
    EnumeratorValueOutOfBounds(String, i64, i64, i64),

    /// Enums must be contain at least one enumerator
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum
    MustContainEnumerators(String),

    /// Enum underlying types must be integral types
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum
    /// * `kind` - The name of the non-integral type that was used as the underlying type of the enum
    UnderlyingTypeMustBeIntegral(String, String),

    // ----------------  Exception Errors ---------------- //
    /// Exceptions cannot be used as a data type with the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    ExceptionNotSupported(Encoding),

    // ----------------  Operation Errors ---------------- //
    /// A streamed parameter was not the last parameter in the operation
    ///
    /// # Fields
    ///
    /// * `parameter_identifier` - The identifier of the parameter that caused the error
    StreamedMembersMustBeLast(String),

    /// The required parameters of an operation did not precede the optional parameters.
    ///
    /// # Fields
    ///
    /// * `parameter_identifier` - The identifier of the parameter that caused the error
    RequiredMustPrecedeOptional(String),

    /// Return tuples for an operation must contain at least two element
    ReturnTuplesMustContainAtLeastTwoElements,

    // ----------------  Struct Errors ---------------- //
    /// Compact structs cannot be empty
    CompactStructCannotBeEmpty,

    /// Compact structs cannot contain tagged data members
    CompactStructCannotContainTaggedMembers,

    // ----------------  Tag Errors ---------------- //
    /// A duplicate tag value was found
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    CannotHaveDuplicateTag(String),

    /// Cannot tag a class
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    CannotTagClass(String),

    /// Cannot tag a member that contains a class
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    CannotTagContainingClass(String),

    /// A tag value was not in the expected range, 0 .. i32::MAX
    TagValueOutOfBounds,

    /// A tagged data member was not set to optional
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    TaggedMemberMustBeOptional(String),

    // ----------------  General Errors ---------------- //
    /// Used to indicate when a method must contain arguments
    ///
    /// # Fields
    ///
    /// * `method_name` - The name of the method
    CannotBeEmpty(&'static str),

    /// Kind can only inherit from a single base
    ///
    /// # Fields
    ///
    /// * `kind` - The kind that can only inherit from a single base
    CanOnlyInheritFromSingleBase(String),
    /// Used to indicate when two concrete types should match, but do not
    ///
    /// # Fields
    ///
    /// * `expected kind` - The name of the expected kind
    /// * `actual kind` - The name of the found kind
    ConcreteTypeMismatch(String, String),

    /// The provided kind should be positive
    ///
    /// # Fields
    ///
    /// * `kind` - The kind that was not positive
    MustBePositive(String),

    /// An identifier was redefined
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier that was redefined
    Redefinition(String),

    /// A self-referential type alias has no concrete type
    ///
    /// # Fields
    ///
    /// * `identifier` - The name of the type alias
    SelfReferentialTypeAliasNeedsConcreteType(String),

    /// An identifier was used to shadow another identifier
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier that is shadowing previously defined identifier
    Shadows(String),

    /// Used to indicate when two types should match, but do not
    ///
    /// # Fields
    ///
    /// * `expected kind` - The name of the expected kind
    /// * `actual kind` - The name of the found kind
    TypeMismatch(String, String),

    // ----------------  SliceC-C# Errors ---------------- //
    // The following are errors that are needed to report cs attribute errors.
    // TODO: Clean up these errors
    AttributeOnlyValidForTopLevelModules(String), // (attribute)
    MissingRequiredArgument(String),              // (arg)
    MissingRequiredAttribute(String),             // (attribute)
    TooManyArguments(String),                     // (expected)
    UnexpectedAttribute(String),                  // (attribute)
}

implement_from_for_error_sub_kind!(LogicErrorKind, DiagnosticKind::LogicError);
implement_error_functions!(
    LogicErrorKind,
    (
        LogicErrorKind::CompressAttributeCannotBeApplied,
        2000,
        "the compress attribute can only be applied to interfaces and operations"
    ),
    (
        LogicErrorKind::DeprecatedAttributeCannotBeApplied,
        2001,
        format!("the deprecated attribute cannot be applied to {kind}"),
        kind
    ),
    (
        LogicErrorKind::CannotBeEmpty,
        2002,
        format!("{method} arguments cannot be empty"),
        method
    ),
    (
        LogicErrorKind::ArgumentNotSupported,
        2003,
        format!("argument '{arg}' is not supported for `{method}`"),
        arg,
        method
    ),
    (
        LogicErrorKind::KeyMustBeNonOptional,
        2004,
        "optional types cannot be used as a dictionary key type"
    ),
    (
        LogicErrorKind::StructKeyMustBeCompact,
        2005,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        LogicErrorKind::KeyTypeNotSupported,
        2006,
        format!("'{identifier}' cannot be used as a dictionary key type"),
        identifier
    ),
    (
        LogicErrorKind::StructKeyContainsDisallowedType,
        2007,
        format!("struct '{identifier}' contains members that cannot be used as a dictionary key type"),
        identifier
    ),
    (
        LogicErrorKind::CannotUseOptionalUnderlyingType,
        2008,
        format!("invalid enum `{}`: enums cannot have optional underlying types", identifier),
        identifier
    ),
    (
        LogicErrorKind::MustContainEnumerators,
        2009,
        format!("invalid enum `{}`: enums must contain at least one enumerator", identifier),
        identifier
    ),
    (
        LogicErrorKind::UnderlyingTypeMustBeIntegral,
        2010,
        format!("invalid enum `{identifier}`: underlying type '{underlying}' is not supported for enums"),
        identifier,
        underlying
    ),
    (
        LogicErrorKind::Redefinition,
        2011,
        format!("redefinition of `{identifier}`"),
        identifier
    ),
    (
        LogicErrorKind::Shadows,
        2012,
        format!("`{identifier}` shadows another symbol"),
        identifier
    ),
    (
        LogicErrorKind::CannotHaveDuplicateTag,
        2000,
        format!("invalid tag on member `{}`: tags must be unique", identifier),
        identifier
    ),
    (
        LogicErrorKind::MustBePositive,
        2013,
        format!("{kind} must be positive"),
        kind
    ),
    (
        LogicErrorKind::RequiredMustPrecedeOptional,
        2015,
        format!("invalid parameter `{}`: required parameters must precede tagged parameters", identifier),
        identifier
    ),
    (
        LogicErrorKind::StreamedMembersMustBeLast,
        2016,
        format!("invalid parameter `{}`: only the last parameter in an operation can use the stream modifier", identifier),
        identifier
    ),
    (
        LogicErrorKind::ReturnTuplesMustContainAtLeastTwoElements,
        2017,
        "return tuples must have at least 2 elements"
    ),
    (
        LogicErrorKind::CompactStructCannotContainTaggedMembers,
        2018,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        LogicErrorKind::TaggedMemberMustBeOptional,
        2019,
        format!("invalid tag on member `{}`: tagged members must be optional", identifier),
        identifier
    ),
    (
        LogicErrorKind::CannotTagClass,
        2020,
        format!("invalid tag on member `{}`: tagged members cannot be classes", identifier),
        identifier
    ),
    (
        LogicErrorKind::CannotTagContainingClass,
        2021,
        format!("invalid tag on member `{}`: tagged members cannot contain classes", identifier),
        identifier
    ),
    (
        LogicErrorKind::CanOnlyInheritFromSingleBase,
        2022,
        format!("`{}` types can only inherit form a single base  {}", kind, kind),
        kind
    ),
    (
        LogicErrorKind::TypeMismatch,
        2023,
        format!("type mismatch: expected a `{expected}` but found a {found} (which doesn't implement `{expected}`)"),
        expected,
        found
    ),
    (
        LogicErrorKind::ConcreteTypeMismatch,
        2024,
        format!("type mismatch: expected `{expected}` but found a `{found}`"),
        expected,
        found
    ),
    (
        LogicErrorKind::CompactStructCannotBeEmpty,
        2025,
        "compact structs must be non-empty"
    ),
    (
        LogicErrorKind::SelfReferentialTypeAliasNeedsConcreteType,
        2026,
        format!("self-referential type alias '{}' has no concrete type", identifier),
        identifier
    ),
    (
        LogicErrorKind::EnumeratorValueOutOfBounds,
        2012,
        format!(
            "invalid enumerator `{identifier}`: enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
        ),
        identifier, value, min, max
    ),
    (
        LogicErrorKind::TagValueOutOfBounds,
        2090,
        "tag values must be within the range 0 <= value <= 2147483647"
    ),
    (
        LogicErrorKind::CannotHaveDuplicateEnumerators,
        2012,
        format!("invalid enumerator `{}`: enumerators must be unique", identifier),
        identifier
    ),
    (
        LogicErrorKind::NotSupportedWithEncoding,
        2026,
        format!("{kind} `{identifier}` is not supported by the {encoding} encoding"),
        kind, identifier, encoding
    ),
    (
        LogicErrorKind::UnsupportedType,
        2026,
        format!("the type `{type_string}` is not supported by the {encoding} encoding"),
        type_string,
        encoding
    ),
    (
        LogicErrorKind::ExceptionNotSupported,
        2026,
        format!("exceptions cannot be used as a data type with the {encoding} encoding"),
        encoding
    ),
    (
        LogicErrorKind::OptionalsNotSupported,
        2026,
        format!("optional types are not supported by the {encoding} encoding (except for classes, proxies, and with tags)"),
        encoding
    ),
    (
        LogicErrorKind::StreamedParametersNotSupported,
        2026,
        format!("streamed parameters are not supported by the {encoding} encoding"),
        encoding
    ),
    (
        LogicErrorKind::UnexpectedAttribute,
        2200,
        format!("unexpected attribute `{attribute}`"),
        attribute
    ),
    (
        LogicErrorKind::MissingRequiredArgument,
        2201,
        format!("missing required argument `{argument}`"),
        argument
    ),
    (
        LogicErrorKind::TooManyArguments,
        2202,
        format!("too many arguments, expected `{expected}`"),
        expected
    ),
    (
        LogicErrorKind::MissingRequiredAttribute,
        2203,
        format!("missing required attribute `{attribute}`"),
        attribute
    ),
    (
        LogicErrorKind::AttributeOnlyValidForTopLevelModules,
        2204,
        format!("The `{attribute}` attribute is only valid for top-level modules"),
        attribute
    )
);
