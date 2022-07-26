// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::{implement_from_for_error_sub_kind, implement_kind_for_enumerator};
pub enum RuleKind {
    CannotBeClass,
    CannotBeEmpty(&'static str),
    CannotContainClasses,
    CannotHaveOptionalUnderlyingType,
    CannotUseOptionalAsKey,
    CanOnlyInheritFromSingleBase,
    CompactStructIsEmpty,
    CompressAttributeCannotBeApplied,
    ConcreteTypeMismatch(String, String),
    DeprecatedAttributeCannotBeApplied(String),
    DuplicateTag,
    ExceptionNotSupported(String),
    MustBeBounded(i64, i64, i64), // (value, min, max)
    MustBeOptional,
    MustBePositive,
    MustBeUnique,
    MustContainAtLeastOneValue,
    NotSupported(String, &'static str),
    NotSupportedInCompactStructs,
    NotSupportedWithEncoding(String, String, String), // (kind, identifier, encoding)
    OptionalsNotSupported(String),
    Redefinition(String),
    RequiredParametersMustBeFirst,
    ReturnTuplesMustContainAtleastTwoElements,
    SelfReferentialTypeAliasNeedsConcreteType(String),
    Shadows(String),
    StreamedParametersNotSupported(String),
    StreamsMustBeLast,
    StructContainsDisallowedType(String),
    StructsMustBeCompactToBeAKey,
    TypeCannotBeUsedAsAKey(String),
    TypeMismatch(String, String),
    UnderlyingTypeMustBeIntegral(String),
    UnsupportedType(String, String), // (type_string, encoding)
}

implement_from_for_error_sub_kind!(RuleKind, ErrorKind::Rule);
implement_kind_for_enumerator!(
    RuleKind,
    (
        RuleKind::CompressAttributeCannotBeApplied,
        2000,
        "the compress attribute can only be applied to interfaces and operations"
    ),
    (
        RuleKind::DeprecatedAttributeCannotBeApplied,
        2001,
        format!("the deprecated attribute cannot be applied to {}", kind).as_str(),
        kind
    ),
    (
        RuleKind::CannotBeEmpty,
        2002,
        format!("{} arguments cannot be empty", method).as_str(),
        method
    ),
    (
        RuleKind::NotSupported,
        2003,
        format!("argument '{}' is not supported for `{}`", arg, method).as_str(),
        arg,
        method
    ),
    (
        RuleKind::CannotUseOptionalAsKey,
        2004,
        "optional types cannot be used as a dictionary key type"
    ),
    (
        RuleKind::StructsMustBeCompactToBeAKey,
        2005,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        RuleKind::TypeCannotBeUsedAsAKey,
        2006,
        format!("'{}' cannot be used as a dictionary key type", identifier).as_str(),
        identifier
    ),
    (
        RuleKind::StructContainsDisallowedType,
        2007,
        format!(
            "struct '{}' contains members that cannot be used as a dictionary key type",
            identifier
        ).as_str(),
        identifier
    ),
    (
        RuleKind::CannotHaveOptionalUnderlyingType,
        2008,
        "enums cannot have optional underlying types"
    ),
    (
        RuleKind::MustContainAtLeastOneValue,
        2009,
        "enums must contain at least one enumerator"
    ),
    (
        RuleKind::UnderlyingTypeMustBeIntegral,
        2010,
        format!("underlying type '{}' is not supported for enums", underlying).as_str(),
        underlying
    ),
    (
        RuleKind::Redefinition,
        2011,
        format!("redefinition of `{}`", identifier).as_str(),
        identifier
    ),
    (
        RuleKind::Shadows,
        2012,
        format!("`{}` shadows another symbol", identifier).as_str(),
        identifier
    ),
    (RuleKind::DuplicateTag, 2000, "tags must be unique"),
    (
        RuleKind::MustBePositive,
        2013,
        "tag values must be positive"
    ),
    (
        RuleKind::MustBeBounded,
        2014,
        "tag values must be greater than or equal to 0 and less than 2147483647"
    ),
    (
        RuleKind::RequiredParametersMustBeFirst,
        2015,
        "required parameters must precede tagged parameters"
    ),
    (
        RuleKind::StreamsMustBeLast,
        2016,
        "only the last parameter in an operation can use the stream modifier"
    ),
    (
        RuleKind::ReturnTuplesMustContainAtleastTwoElements,
        2017,
        "return tuples must have at least 2 elements"
    ),
    (
        RuleKind::NotSupportedInCompactStructs,
        2018,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        RuleKind::MustBeOptional,
        2019,
        "tagged members must be optional"
    ),
    (
        RuleKind::CannotBeClass,
        2020,
        "tagged members cannot be classes"
    ),
    (
        RuleKind::CannotContainClasses,
        2021,
        "tagged members cannot contain classes"
    ),
    (
        RuleKind::CanOnlyInheritFromSingleBase,
        2022,
        "exceptions can only inherit from a single base exception"
    ),
    (
        RuleKind::TypeMismatch,
        2023,
        format!(
            "type mismatch: expected a `{}` but found {} (which doesn't implement `{}`)",
            expected, found, expected
        ).as_str(),
        expected,
        found
    ),
    (
        RuleKind::ConcreteTypeMismatch,
        2024,
        format!("type mismatch: expected `{}` but found `{}`", expected, found).as_str(),
        expected,
        found
    ),
    (
        RuleKind::CompactStructIsEmpty,
        2025,
        "compact structs must be non-empty"
    ),
    (
        RuleKind::SelfReferentialTypeAliasNeedsConcreteType,
        2026,
        format!("self-referential type alias '{}' has no concrete type", identifier).as_str(),
        identifier
    ),
    (
        RuleKind::MustBePositive,
        2011,
        "enumerators must be non-negative"
    ),
    (
        RuleKind::MustBeBounded,
        2012,
        format!(
            "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
            value = value,
            min = min,
            max = max
        ).as_str(),
        value,
        min,
        max
    ),
    (
        RuleKind::MustBeUnique,
        2012,
        "enumerators must be unique"
    ),
    (
        RuleKind::NotSupported,
        2026,
        format!(
            "{} `{}` is not supported by the Slice{} encoding",
            kind, identifier, encoding,
        ).as_str(),
        kind,
        identifier,
        encoding
    ),
    (
        RuleKind::UnsupportedType,
        2026,
        format!(
            "the type `{}` is not supported by the Slice{} encoding",
            type_string, encoding,
        ).as_str(),
        type_string,
        encoding
    ),
    (
        RuleKind::ExceptionNotSupported,
        2026,
        format!(
            "exceptions cannot be used as a data type with the Slice{} encoding",
            encoding
        ).as_str(),
        encoding
    ),
    (
        RuleKind::OptionalsNotSupported,
        2026,
        format!(
            "optional types are not supported by the {} encoding (except for classes, proxies, and with tags)",
            encoding
        ).as_str(),
        encoding
    ),
    (
        RuleKind::StreamedParametersNotSupported,
        2026,
        format!("streamed parameters are not supported by the {} encoding", encoding).as_str(),
        encoding
    )
);
