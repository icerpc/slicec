// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::grammar::Encoding;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum LogicKind {
    CannotBeClass,
    CannotBeEmpty(&'static str),
    CannotContainClasses,
    CannotHaveOptionalUnderlyingType,
    CannotUseOptionalAsKey,
    CanOnlyInheritFromSingleBase,
    CompactStructIsEmpty,
    CompressAttributeCannotBeApplied,
    ConcreteTypeMismatch(String, String),
    ClassesCanOnlyInheritFromSingleBase,
    DeprecatedAttributeCannotBeApplied(String),
    DuplicateTag,
    ExceptionNotSupported(Encoding),
    MustBeBounded(i64, i64, i64), // (value, min, max)
    MustBeInI32Range,
    MustBeOptional,
    MustBePositive(String), // (kind)
    MustBeUnique,
    MustContainAtLeastOneValue,
    ArgumentNotSupported(String, String), // (arg, method)
    NotSupportedInCompactStructs,
    NotSupportedWithEncoding(String, String, Encoding), // (kind, identifier, encoding)
    OptionalsNotSupported(Encoding),
    Redefinition(String),
    RequiredParametersMustBeFirst,
    ReturnTuplesMustContainAtLeastTwoElements,
    SelfReferentialTypeAliasNeedsConcreteType(String),
    Shadows(String),
    StreamedParametersNotSupported(Encoding),
    StreamsMustBeLast,
    StructContainsDisallowedType(String),
    StructsMustBeCompactToBeAKey,
    TagOutOfBounds,
    TypeCannotBeUsedAsAKey(String),
    TypeMismatch(String, String),
    UnderlyingTypeMustBeIntegral(String),
    UnsupportedType(String, Encoding), // (type_string, encoding)
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
        LogicKind::MustBeInI32Range,
        2014,
        "tag values must be greater than or equal to 0 and less than 2147483647"
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
    )
);
