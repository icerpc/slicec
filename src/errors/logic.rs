// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::ErrorKind;
use crate::grammar::Encoding;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum LogicKind {
    // ----------------  Attribute Errors ---------------- //
    /// Used to indicate when the compress attribute cannot be applied
    CompressAttributeCannotBeApplied,

    /// Used to indicate when the deprecated attribute cannot be applied
    ///
    /// # Fields
    ///
    /// * `type` - The type which the deprecated attribute was applied to
    DeprecatedAttributeCannotBeApplied(String),

    // ----------------  Argument Errors ---------------- //
    /// The provided argument is not supported for the given method
    ///
    /// # Fields
    ///
    /// * `argument_name` - The name of the argument
    /// * `method_name` - The name of the method
    ArgumentNotSupported(String, String),

    // ----------------  Tag Errors ---------------- //
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

    /// A duplicate tag value was found
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    CannotHaveDuplicateTag(String),

    /// A tag value was not in the expected range, 0 .. i32::MAX
    TagValueOutOfBounds,

    /// A tagged data member was not set to optional
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member
    TaggedMemberMustBeOptional(String),

    // ----------------  Enum Errors ---------------- //
    /// Enums cannot have optional underlying types
    CannotUseOptionalUnderlyingType,

    /// Enums must be contain at least one enumerator
    MustContainEnumerators,

    /// Enumerators must be unique
    CannotHaveDuplicateEnumerators,

    /// Enum underlying types must be integral types
    ///
    /// # Fields
    ///
    /// * `type` - The name of the non-integral type that was used as the underlying type of the enum
    UnderlyingTypeMustBeIntegral(String),

    /// An enumerator was found that was out of bounds of the underlying type of the parent enum
    ///
    /// # Fields
    ///
    /// * `value` - The value of the out of bounds enumerator
    /// * `min` - The minimum value of the underlying type of the enum
    /// * `max` - The maximum value of the underlying type of the enum
    EnumeratorValueOutOfBounds(i64, i64, i64),

    // ---------------- Dictionary Errors ---------------- //
    /// Dictionaries cannot use optional types as keys
    KeyMustBeNonOptional,

    /// Struct contains a member that cannot be used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `struct_identifier` - The identifier of the struct
    StructKeyContainsDisallowedType(String),

    /// Structs must be compact to be used as a dictionary key type
    StructKeyMustBeCompact,

    /// An unsupported type was used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the type that was used as a dictionary key type
    KeyTypeNotSupported(String),

    // ----------------  Struct Errors ---------------- //
    /// Compact structs cannot be empty
    CompactStructCannotBeEmpty,

    /// Compact structs cannot contain tagged data members
    CompactStructCannotContainTaggedMembers,

    // ----------------  Exception Errors ---------------- //
    /// Exceptions cannot be used as a data type with the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    ExceptionNotSupported(Encoding),

    // ----------------  Operation Errors ---------------- //
    /// The required parameters of an operation did not precede the optional parameters.
    RequiredMustPrecedeOptional,

    /// Return tuples for an operation must contain at least two element
    ReturnTuplesMustContainAtLeastTwoElements,

    /// A streamed parameter was not the last parameter in the operation
    StreamedMembersMustBeLast,

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
    /// * `type` - The name of the type that was used in the specified encoding
    /// * `encoding` - The encoding that was specified
    UnsupportedType(String, Encoding),

    // ----------------  General Errors ---------------- //
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
    /// * `expected type` - The name of the expected type
    /// * `actual type` - The name of the found type
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
    /// * `expected type` - The name of the expected type
    /// * `actual type` - The name of the found type
    TypeMismatch(String, String),

    /// Used to indicate when a method must contain arguments
    ///
    /// # Fields
    ///
    /// * `method_name` - The name of the method
    CannotBeEmpty(&'static str),

    // ----------------  SliceC-C# Errors ---------------- //
    // The following are errors that are needed to report cs attribute errors.
    // TODO: Clean up these errors
    UnexpectedAttribute(String),                  // (attribute)
    MissingRequiredArgument(String),              // (arg)
    TooManyArguments(String),                     // (expected)
    MissingRequiredAttribute(String),             // (attribute)
    AttributeOnlyValidForTopLevelModules(String), // (attribute)
}

implement_from_for_error_sub_kind!(LogicKind, ErrorKind::Logic);
implement_error_functions!(
    LogicKind,
    (
        LogicKind::CompressAttributeCannotBeApplied,
        2000,
        "the compress attribute can only be applied to interfaces and operations"
    ),
    (
        LogicKind::DeprecatedAttributeCannotBeApplied,
        2001,
        format!("the deprecated attribute cannot be applied to {}", kind),
        kind
    ),
    (
        LogicKind::CannotBeEmpty,
        2002,
        format!("{} arguments cannot be empty", method),
        method
    ),
    (
        LogicKind::ArgumentNotSupported,
        2003,
        format!("argument '{}' is not supported for `{}`", arg, method),
        arg,
        method
    ),
    (
        LogicKind::KeyMustBeNonOptional,
        2004,
        "optional types cannot be used as a dictionary key type"
    ),
    (
        LogicKind::StructKeyMustBeCompact,
        2005,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        LogicKind::KeyTypeNotSupported,
        2006,
        format!("'{}' cannot be used as a dictionary key type", identifier),
        identifier
    ),
    (
        LogicKind::StructKeyContainsDisallowedType,
        2007,
        format!(
            "struct '{}' contains members that cannot be used as a dictionary key type",
            identifier
        ),
        identifier
    ),
    (
        LogicKind::CannotUseOptionalUnderlyingType,
        2008,
        "enums cannot have optional underlying types"
    ),
    (
        LogicKind::MustContainEnumerators,
        2009,
        "enums must contain at least one enumerator"
    ),
    (
        LogicKind::UnderlyingTypeMustBeIntegral,
        2010,
        format!("underlying type '{}' is not supported for enums", underlying),
        underlying
    ),
    (
        LogicKind::Redefinition,
        2011,
        format!("redefinition of `{}`", identifier),
        identifier
    ),
    (
        LogicKind::Shadows,
        2012,
        format!("`{}` shadows another symbol", identifier),
        identifier
    ),
    (
        LogicKind::CannotHaveDuplicateTag,
        2000,
        format!("invalid tag on member `{}`: tags must be unique", identifier),
        identifier
    ),
    (
        LogicKind::MustBePositive,
        2013,
        format!("{kind} must be positive"),
        kind
    ),
    (
        LogicKind::RequiredMustPrecedeOptional,
        2015,
        "required parameters must precede tagged parameters"
    ),
    (
        LogicKind::StreamedMembersMustBeLast,
        2016,
        "only the last parameter in an operation can use the stream modifier"
    ),
    (
        LogicKind::ReturnTuplesMustContainAtLeastTwoElements,
        2017,
        "return tuples must have at least 2 elements"
    ),
    (
        LogicKind::CompactStructCannotContainTaggedMembers,
        2018,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        LogicKind::TaggedMemberMustBeOptional,
        2019,
        format!("invalid tag on member `{}`: tagged members must be optional", identifier),
        identifier
    ),
    (
        LogicKind::CannotTagClass,
        2020,
        format!("invalid tag on member `{}`: tagged members cannot be classes", identifier),
        identifier
    ),
    (
        LogicKind::CannotTagContainingClass,
        2021,
        format!("invalid tag on member `{}`: tagged members cannot contain classes", identifier),
        identifier
    ),
    (
        LogicKind::CanOnlyInheritFromSingleBase,
        2022,
        format!("`{}` can only inherit from a single base {}", kind, kind),
        kind
    ),
    (
        LogicKind::TypeMismatch,
        2023,
        format!(
            "type mismatch: expected a `{}` but found {} (which doesn't implement `{}`)",
            expected, found, expected
        ),
        expected,
        found
    ),
    (
        LogicKind::ConcreteTypeMismatch,
        2024,
        format!("type mismatch: expected `{}` but found `{}`", expected, found),
        expected,
        found
    ),
    (
        LogicKind::CompactStructCannotBeEmpty,
        2025,
        "compact structs must be non-empty"
    ),
    (
        LogicKind::SelfReferentialTypeAliasNeedsConcreteType,
        2026,
        format!("self-referential type alias '{}' has no concrete type", identifier),
        identifier
    ),
    (
        LogicKind::EnumeratorValueOutOfBounds,
        2012,
        format!(
            "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
            value = value,
            min = min,
            max = max
        ),
        value,
        min,
        max
    ),
    (
        LogicKind::TagValueOutOfBounds,
        2090,
        "tag values must be within the range 0 <= value <= 2147483647"
    ),
    (
        LogicKind::CannotHaveDuplicateEnumerators,
        2012,
        "enumerators must be unique"
    ),
    (
        LogicKind::NotSupportedWithEncoding,
        2026,
        format!(
            "{} `{}` is not supported by the {} encoding",
            kind, identifier, encoding,
        ),
        kind,
        identifier,
        encoding
    ),
    (
        LogicKind::UnsupportedType,
        2026,
        format!(
            "the type `{}` is not supported by the {} encoding",
            type_string, encoding,
        ),
        type_string,
        encoding
    ),
    (
        LogicKind::ExceptionNotSupported,
        2026,
        format!(
            "exceptions cannot be used as a data type with the {} encoding",
            encoding
        ),
        encoding
    ),
    (
        LogicKind::OptionalsNotSupported,
        2026,
        format!(
            "optional types are not supported by the {} encoding (except for classes, proxies, and with tags)",
            encoding
        ),
        encoding
    ),
    (
        LogicKind::StreamedParametersNotSupported,
        2026,
        format!("streamed parameters are not supported by the {} encoding", encoding),
        encoding
    ),
    (
        LogicKind::UnexpectedAttribute,
        2200,
        format!("unexpected attribute `{}`", attribute),
        attribute
    ),
    (
        LogicKind::MissingRequiredArgument,
        2201,
        format!("missing required argument `{}`", argument),
        argument
    ),
    (
        LogicKind::TooManyArguments,
        2202,
        format!("too many arguments, expected `{}`", expected),
        expected
    ),
    (
        LogicKind::MissingRequiredAttribute,
        2203,
        format!("missing required attribute `{}`", attribute),
        attribute
    ),
    (
        LogicKind::AttributeOnlyValidForTopLevelModules,
        2204,
        format!("The `{}` attribute is only valid for top-level modules", attribute),
        attribute
    )
);
