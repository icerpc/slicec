// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::ErrorKind;
use crate::grammar::Encoding;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum LogicKind {
    /// Cannot tag a class
    CannotBeClass,

    /// Used to indicate when a method must contain arguments
    ///
    /// # Fields
    ///
    /// * `method_name` - The name of the method
    CannotBeEmpty(&'static str),

    /// Cannot tag a member that contains a class
    CannotContainClasses,

    /// Enums cannot have optional underlying types
    CannotHaveOptionalUnderlyingType,

    /// Dictionaries cannot use optional types as keys
    CannotUseOptionalAsKey,

    /// Exceptions can only inherit from a single base exception
    CanOnlyInheritFromSingleBase,

    /// Compact structs cannot be empty
    CompactStructIsEmpty,

    /// Used to indicate when the compress attribute cannot be applied
    CompressAttributeCannotBeApplied,

    /// Used to indicate when two concrete types should match, but do not
    ///
    /// # Fields
    ///
    /// * `expected type` - The name of the expected type
    /// * `actual type` - The name of the found type
    ConcreteTypeMismatch(String, String),

    /// Classes can only inherit from a single base class
    ClassesCanOnlyInheritFromSingleBase,

    /// Used to indicate when the deprecated attribute cannot be applied
    ///
    /// # Fields
    ///
    /// * `type` - The type which the deprecated attribute was applied to
    DeprecatedAttributeCannotBeApplied(String),

    /// A duplicate tag value was found
    DuplicateTag,

    /// Exceptions cannot be used as a data type with the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    ExceptionNotSupported(Encoding),

    /// An enumerator was found that was out of bounds of the underlying type of the parent enum
    ///
    /// # Fields
    ///
    /// * `value` - The value of the out of bounds enumerator
    /// * `min` - The minimum value of the underlying type of the enum
    /// * `max` - The maximum value of the underlying type of the enum
    MustBeBounded(i64, i64, i64),

    /// A tagged data member was not set to optional
    MustBeOptional,

    /// The provided kind should be positive
    ///
    /// # Fields
    ///
    /// * `kind` - The kind that was not positive
    MustBePositive(String),

    /// Enumerators must be unique
    MustBeUnique,

    /// Enums must be contain at least one enumerator
    MustContainAtLeastOneValue,

    /// The provided argument is not supported for the given method
    ///
    /// # Fields
    ///
    /// * `argument_name` - The name of the argument
    /// * `method_name` - The name of the method
    ArgumentNotSupported(String, String),

    /// Compact structs cannot contain tagged data members
    NotSupportedInCompactStructs,

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

    /// An identifier was redefined
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier that was redefined
    Redefinition(String),

    /// The required parameters of an operation did not precede the optional parameters.
    RequiredParametersMustBeFirst, // TODO: Perhaps this should be a warning?

    /// Return tuples for an operation must contain at least two element
    ReturnTuplesMustContainAtLeastTwoElements, // TODO: Perhaps this should be a warning?

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

    /// Streamed parameters are not supported with the specified encoding
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified
    StreamedParametersNotSupported(Encoding),

    /// A streamed parameter was not the last parameter in the operation
    StreamsMustBeLast,

    /// Struct contains a member that cannot be used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `struct_identifier` - The identifier of the struct
    StructContainsDisallowedType(String),

    /// Structs must be compact to be used as a dictionary key type
    StructsMustBeCompactToBeAKey,

    /// A tag value was not in the expected range, 0 .. i32::MAX
    TagOutOfBounds,

    /// An unsupported type was used as a dictionary key type
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the type that was used as a dictionary key type
    TypeCannotBeUsedAsAKey(String),

    /// Used to indicate when two types should match, but do not
    ///
    /// # Fields
    ///
    /// * `expected type` - The name of the expected type
    /// * `actual type` - The name of the found type
    TypeMismatch(String, String),

    /// Enum underlying types must be integral types
    ///
    /// # Fields
    ///
    /// * `type` - The name of the non-integral type that was used as the underlying type of the enum
    UnderlyingTypeMustBeIntegral(String),

    /// An unsupported type was used in the specified encoding
    ///
    /// # Fields
    ///
    /// * `type` - The name of the type that was used in the specified encoding
    /// * `encoding` - The encoding that was specified
    UnsupportedType(String, Encoding),

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
        LogicKind::CannotUseOptionalAsKey,
        2004,
        "optional types cannot be used as a dictionary key type"
    ),
    (
        LogicKind::StructsMustBeCompactToBeAKey,
        2005,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        LogicKind::TypeCannotBeUsedAsAKey,
        2006,
        format!("'{}' cannot be used as a dictionary key type", identifier),
        identifier
    ),
    (
        LogicKind::StructContainsDisallowedType,
        2007,
        format!(
            "struct '{}' contains members that cannot be used as a dictionary key type",
            identifier
        ),
        identifier
    ),
    (
        LogicKind::CannotHaveOptionalUnderlyingType,
        2008,
        "enums cannot have optional underlying types"
    ),
    (
        LogicKind::MustContainAtLeastOneValue,
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
    (LogicKind::DuplicateTag, 2000, "tags must be unique"),
    (
        LogicKind::MustBePositive,
        2013,
        format!("{kind} must be positive"),
        kind
    ),
    (
        LogicKind::RequiredParametersMustBeFirst,
        2015,
        "required parameters must precede tagged parameters"
    ),
    (
        LogicKind::StreamsMustBeLast,
        2016,
        "only the last parameter in an operation can use the stream modifier"
    ),
    (
        LogicKind::ReturnTuplesMustContainAtLeastTwoElements,
        2017,
        "return tuples must have at least 2 elements"
    ),
    (
        LogicKind::NotSupportedInCompactStructs,
        2018,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        LogicKind::MustBeOptional,
        2019,
        "tagged members must be optional"
    ),
    (
        LogicKind::CannotBeClass,
        2020,
        "tagged members cannot be classes"
    ),
    (
        LogicKind::CannotContainClasses,
        2021,
        "tagged members cannot contain classes"
    ),
    (
        LogicKind::CanOnlyInheritFromSingleBase,
        2022,
        "exceptions can only inherit from a single base exception"
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
        LogicKind::CompactStructIsEmpty,
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
        LogicKind::MustBeBounded,
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
        LogicKind::TagOutOfBounds,
        2090,
        "tag values must be greater than or equal to 0 and less than 2147483647"
    ),
    (
        LogicKind::MustBeUnique,
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
        LogicKind::ClassesCanOnlyInheritFromSingleBase,
        2027,
        "classes can only inherit from a single base class"
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
