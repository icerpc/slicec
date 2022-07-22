// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorLevel;
use crate::errors::*;

#[derive(Debug, Clone)]
pub enum RuleKind {
    InvalidAttribute(InvalidAttributeKind),
    InvalidArgument(InvalidArgumentKind),
    InvalidTag(String, InvalidTagKind),
    InvalidParameter(String, InvalidParameterKind),
    InvalidMember(String, InvalidMemberKind),
    InvalidEnum(String, InvalidEnumKind),
    InvalidEnumerator(String, InvalidEnumeratorKind),
    InvalidEncoding(InvalidEncodingKind),
    InvalidException(InvalidExceptionKind),
    InvalidStruct(InvalidStructKind),
    InvalidIdentifier(InvalidIdentifierKind),
    InvalidTypeAlias(InvalidTypeAliasKind),
    InvalidKey(InvalidKeyKind),
    InvalidType(InvalidTypeKind),
}

impl ErrorType for RuleKind {
    fn error_code(&self) -> u32 {
        match &self {
            RuleKind::InvalidAttribute(kind) => kind.error_code(),
            RuleKind::InvalidArgument(kind) => kind.error_code(),
            RuleKind::InvalidEncoding(kind) => kind.error_code(),
            RuleKind::InvalidEnum(_, kind) => kind.error_code(),
            RuleKind::InvalidEnumerator(_, kind) => kind.error_code(),
            RuleKind::InvalidIdentifier(kind) => kind.error_code(),
            RuleKind::InvalidKey(kind) => kind.error_code(),
            RuleKind::InvalidParameter(_, kind) => kind.error_code(),
            RuleKind::InvalidStruct(kind) => kind.error_code(),
            RuleKind::InvalidTag(_, kind) => kind.error_code(),
            RuleKind::InvalidTypeAlias(kind) => kind.error_code(),
            RuleKind::InvalidMember(_, kind) => kind.error_code(),
            RuleKind::InvalidException(kind) => kind.error_code(),
            RuleKind::InvalidType(kind) => kind.error_code(),
        }
    }

    fn message(&self) -> String {
        match self {
            RuleKind::InvalidArgument(arg_kind) => "[InvalidArgument]: ".to_owned() + &arg_kind.get_description(),
            RuleKind::InvalidKey(key_kind) => "[InvalidKey]: ".to_owned() + &key_kind.get_description(),
            RuleKind::InvalidEnum(id, kind) => format!("[InvalidEnum `{}`]: ", id) + &kind.get_description(),
            RuleKind::InvalidEncoding(kind) => "[InvalidEncoding]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidIdentifier(kind) => "[InvalidIdentifier]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidException(kind) => "[InvalidException]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidType(kind) => "[InvalidType]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidAttribute(kind) => "[InvalidAttribute]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidTag(tag, kind) => format!("[InvalidTag on `{}`]: ", tag) + &kind.get_description(),
            RuleKind::InvalidEnumerator(identifier, kind) => {
                format!("[InvalidEnumerator `{}`]: ", identifier) + &kind.get_description()
            }
            RuleKind::InvalidParameter(identifier, kind) => {
                format!("[InvalidParameter `{}`]: ", identifier) + &kind.get_description()
            }
            RuleKind::InvalidMember(identifier, kind) => {
                format!("[InvalidMember `{}`]: ", identifier) + &kind.get_description()
            }
            _ => "TODO".to_owned(),
        }
    }

    fn severity(&self) -> ErrorLevel {
        ErrorLevel::Error
    }
}

#[derive(Debug, Clone)]
pub enum InvalidAttributeKind {
    CompressAttributeCannotBeApplied,
    DeprecatedAttributeCannotBeApplied(String),
}

#[derive(Debug, Clone)]
pub enum InvalidArgumentKind {
    CannotBeEmpty(&'static str),
    NotSupported(String, &'static str),
}

#[derive(Debug, Clone)]
pub enum InvalidKeyKind {
    CannotUseOptionalAsKey,
    StructsMustBeCompactToBeAKey,
    TypeCannotBeUsedAsAKey(String),
    StructContainsDisallowedType(String),
}

#[derive(Debug, Clone)]
pub enum InvalidEnumKind {
    CannotHaveOptionalUnderlyingType,
    MustContainAtLeastOneValue,
    UnderlyingTypeMustBeIntegral(String),
}

#[derive(Debug, Clone)]
pub enum InvalidEnumeratorKind {
    MustBePositive,
    MustBeBounded(i64, i64, i64), // (value, min, max)
    MustBeUnique,
}

#[derive(Debug, Clone)]
pub enum InvalidIdentifierKind {
    Redefinition(String),
    Shadows(String),
}

#[derive(Debug, Clone)]
pub enum InvalidTagKind {
    DuplicateTag,
    MustBePositive,
    MustBeBounded,
}

#[derive(Debug, Clone)]
pub enum InvalidParameterKind {
    RequiredParametersMustBeFirst,
    StreamsMustBeLast,
    ReturnTuplesMustContainAtleastTwoElements,
}

#[derive(Debug, Clone)]
pub enum InvalidMemberKind {
    NotSupportedInCompactStructs,
    MustBeOptional,
    CannotBeClass,
    CannotContainClasses,
}

#[derive(Debug, Clone)]
pub enum InvalidExceptionKind {
    CanOnlyInheritFromSingleBase,
}

#[derive(Debug, Clone)]
pub enum InvalidTypeKind {
    TypeMismatch(String, String),
    ConcreteTypeMismatch(String, String),
}

#[derive(Debug, Clone)]
pub enum InvalidStructKind {
    CompactStructIsEmpty,
}

#[derive(Debug, Clone)]
pub enum InvalidTypeAliasKind {
    SelfReferentialTypeAliasNeedsConcreteType(String),
}

#[derive(Debug, Clone)]
pub enum InvalidEncodingKind {
    NotSupported(String, String, String), // (kind, identifier, encoding)
    UnsupportedType(String, String),      // (type_string, encoding)
    ExceptionNotSupported(String),
    OptionalsNotSupported(String),
    StreamedParametersNotSupported(String),
}

macro_rules! implement_kind_for_enumerator {
    ($enumerator:ty, $(($kind:path , $code:expr, $message:expr $(, $args:pat)*)),*) => {
        impl $enumerator {
            pub fn error_code(&self) -> u32 {
                match self {
                    $(
                        $kind$(($args))* => $code,
                    )*
                }
            }
            pub fn get_description(&self) -> String {
                match self {
                    $(
                        $kind$(($args))* => $message,
                    )*
                }
            }
        }
    };
}

implement_kind_for_enumerator!(
    InvalidAttributeKind,
    (
        InvalidAttributeKind::CompressAttributeCannotBeApplied,
        2000,
        "the compress attribute can only be applied to interfaces and operations".to_owned()
    ),
    (
        InvalidAttributeKind::DeprecatedAttributeCannotBeApplied,
        2001,
        format!("the deprecated attribute cannot be applied to {}", kind),
        kind
    )
);
implement_kind_for_enumerator!(
    InvalidArgumentKind,
    (
        InvalidArgumentKind::CannotBeEmpty,
        2002,
        format!("{} arguments cannot be empty", method),
        method
    ),
    (
        InvalidArgumentKind::NotSupported,
        2003,
        format!("argument '{}' is not supported for `{}`", arg, method),
        arg,
        method
    )
);
implement_kind_for_enumerator!(
    InvalidKeyKind,
    (
        InvalidKeyKind::CannotUseOptionalAsKey,
        2004,
        "optional types cannot be used as a dictionary key type".to_owned()
    ),
    (
        InvalidKeyKind::StructsMustBeCompactToBeAKey,
        2005,
        "structs must be compact to be used as a dictionary key type".to_owned(),
        method
    ),
    (
        InvalidKeyKind::TypeCannotBeUsedAsAKey,
        2006,
        format!("'{}' cannot be used as a dictionary key type", identifier),
        identifier
    ),
    (
        InvalidKeyKind::StructContainsDisallowedType,
        2007,
        format!(
            "struct '{}' contains members that cannot be used as a dictionary key type",
            identifier
        ),
        identifier
    )
);
implement_kind_for_enumerator!(
    InvalidEnumKind,
    (
        InvalidEnumKind::CannotHaveOptionalUnderlyingType,
        2008,
        "enums cannot have optional underlying types".to_owned()
    ),
    (
        InvalidEnumKind::MustContainAtLeastOneValue,
        2009,
        "enums must contain at least one enumerator".to_owned()
    ),
    (
        InvalidEnumKind::UnderlyingTypeMustBeIntegral,
        2010,
        format!("underlying type '{}' is not supported for enums", underlying),
        underlying
    )
);
implement_kind_for_enumerator!(
    InvalidIdentifierKind,
    (
        InvalidIdentifierKind::Redefinition,
        2011,
        format!("redefinition of `{}`", identifier),
        identifier
    ),
    (
        InvalidIdentifierKind::Shadows,
        2012,
        format!("`{}` shadows another symbol", identifier),
        identifier
    )
);
implement_kind_for_enumerator!(
    InvalidTagKind,
    (InvalidTagKind::DuplicateTag, 2000, "tags must be unique".to_owned()),
    (
        InvalidTagKind::MustBePositive,
        2013,
        "tag values must be positive".to_owned()
    ),
    (
        InvalidTagKind::MustBeBounded,
        2014,
        "tag values must be greater than or equal to 0 and less than 2147483647".to_owned()
    )
);
implement_kind_for_enumerator!(
    InvalidParameterKind,
    (
        InvalidParameterKind::RequiredParametersMustBeFirst,
        2015,
        "required parameters must precede tagged parameters".to_owned()
    ),
    (
        InvalidParameterKind::StreamsMustBeLast,
        2016,
        "only the last parameter in an operation can use the stream modifier".to_owned()
    ),
    (
        InvalidParameterKind::ReturnTuplesMustContainAtleastTwoElements,
        2017,
        "return tuples must have at least 2 elements".to_owned()
    )
);
implement_kind_for_enumerator!(
    InvalidMemberKind,
    (
        InvalidMemberKind::NotSupportedInCompactStructs,
        2018,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact".to_owned()
    ),
    (
        InvalidMemberKind::MustBeOptional,
        2019,
        "tagged members must be optional".to_owned()
    ),
    (
        InvalidMemberKind::CannotBeClass,
        2020,
        "tagged members cannot be classes".to_owned()
    ),
    (
        InvalidMemberKind::CannotContainClasses,
        2021,
        "tagged members cannot contain classes".to_owned()
    )
);
implement_kind_for_enumerator!(
    InvalidExceptionKind,
    (
        InvalidExceptionKind::CanOnlyInheritFromSingleBase,
        2022,
        "exceptions can only inherit from a single base exception".to_owned()
    )
);
implement_kind_for_enumerator!(
    InvalidTypeKind,
    (
        InvalidTypeKind::TypeMismatch,
        2023,
        format!(
            "type mismatch: expected a `{}` but found {} (which doesn't implement `{}`)",
            expected, found, expected
        ),
        expected,
        found
    ),
    (
        InvalidTypeKind::ConcreteTypeMismatch,
        2024,
        format!("type mismatch: expected `{}` but found `{}`", expected, found),
        expected,
        found
    )
);
implement_kind_for_enumerator!(
    InvalidStructKind,
    (
        InvalidStructKind::CompactStructIsEmpty,
        2025,
        "compact structs must be non-empty".to_owned()
    )
);
implement_kind_for_enumerator!(
    InvalidTypeAliasKind,
    (
        InvalidTypeAliasKind::SelfReferentialTypeAliasNeedsConcreteType,
        2026,
        format!("self-referential type alias '{}' has no concrete type", identifier),
        identifier
    )
);
implement_kind_for_enumerator!(
    InvalidEnumeratorKind,
    (
        InvalidEnumeratorKind::MustBePositive,
        2011,
        "enumerators must be non-negative".to_owned()
    ),
    (
        InvalidEnumeratorKind::MustBeBounded,
        2012,
        format!(
            "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
            value, min, max,
        ),
        value,
        min,
        max
    ),
    (
        InvalidEnumeratorKind::MustBeUnique,
        2012,
        "enumerators must be unique".to_owned()
    )
);

implement_kind_for_enumerator!(
    InvalidEncodingKind,
    (
        InvalidEncodingKind::NotSupported,
        2026,
        format!(
            "{} `{}` is not supported by the Slice{} encoding",
            kind, identifier, encoding,
        ),
        kind,
        identifier,
        encoding
    ),
    (
        InvalidEncodingKind::UnsupportedType,
        2026,
        format!(
            "the type `{}` is not supported by the Slice{} encoding",
            type_string, encoding,
        ),
        type_string,
        encoding
    ),
    (
        InvalidEncodingKind::ExceptionNotSupported,
        2026,
        format!(
            "exceptions cannot be used as a data type with the Slice{} encoding",
            encoding
        ),
        encoding
    ),
    (
        InvalidEncodingKind::OptionalsNotSupported,
        2026,
        format!(
            "optional types are not supported by the {} encoding (except for classes, proxies, and with tags)",
            encoding
        ),
        encoding
    ),
    (
        InvalidEncodingKind::StreamedParametersNotSupported,
        2026,
        format!("streamed parameters are not supported by the {} encoding", encoding),
        identifier
    )
);
macro_rules! implement_from_for_rule_kind {
    ($type:ty, $enumerator:path) => {
        impl From<$type> for RuleKind {
            fn from(original: $type) -> RuleKind {
                $enumerator(original)
            }
        }
    };
}

implement_from_for_rule_kind!(InvalidAttributeKind, RuleKind::InvalidAttribute);
implement_from_for_rule_kind!(InvalidKeyKind, RuleKind::InvalidKey);
implement_from_for_rule_kind!(InvalidArgumentKind, RuleKind::InvalidArgument);
implement_from_for_rule_kind!(InvalidIdentifierKind, RuleKind::InvalidIdentifier);
implement_from_for_rule_kind!(InvalidExceptionKind, RuleKind::InvalidException);
implement_from_for_rule_kind!(InvalidTypeKind, RuleKind::InvalidType);
implement_from_for_rule_kind!(InvalidStructKind, RuleKind::InvalidStruct);
implement_from_for_rule_kind!(InvalidEncodingKind, RuleKind::InvalidEncoding);
implement_from_for_rule_kind!(InvalidTypeAliasKind, RuleKind::InvalidTypeAlias);
