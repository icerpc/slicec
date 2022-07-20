// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorLevel;
use crate::errors::*;

#[derive(Debug, Clone)]
pub enum RuleKind {
    InvalidAttribute(InvalidAttributeKind),
    InvalidArgument(InvalidArgumentKind),
    InvalidTag(String, InvalidTagKind),
    InvalidParameter(InvalidParameterKind),
    InvalidMember(String, InvalidMemberKind),
    InvalidEnum(String, InvalidEnumKind),
    InvalidEnumerator {
        identifier: String,
        kind: InvalidEnumeratorKind,
    },
    InvalidEncoding(InvalidEncodingKind),
    InvalidException(InvalidExceptionKind),
    InvalidStruct(String, InvalidStructKind),
    InvalidIdentifier(InvalidIdentifierKind),
    InvalidTypeAlias(InvalidTypeAliasKind),
    InvalidKey(InvalidKeyKind),
    InvalidType(InvalidTypeKind),
}

impl ErrorType for RuleKind {
    fn error_code(&self) -> u32 {
        match &self {
            RuleKind::InvalidAttribute(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidArgument(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidEncoding(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidEnum(_, kind) => 2000 + kind.error_code(),
            RuleKind::InvalidEnumerator { identifier: _, kind } => 2000 + kind.error_code(),
            RuleKind::InvalidIdentifier(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidKey(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidParameter(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidStruct(_, kind) => 2000 + kind.error_code(),
            RuleKind::InvalidTag(_, kind) => 2000 + kind.error_code(),
            RuleKind::InvalidTypeAlias(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidMember(_, kind) => 2000 + kind.error_code(),
            RuleKind::InvalidException(kind) => 2000 + kind.error_code(),
            RuleKind::InvalidType(kind) => 2000 + kind.error_code(),
        }
    }

    fn message(&self) -> String {
        match self {
            RuleKind::InvalidAttribute(attribute_kind) => {
                "[InvalidAttribute]: ".to_owned() + &attribute_kind.get_description()
            }
            RuleKind::InvalidArgument(arg_kind) => "[InvalidArgument]: ".to_owned() + &arg_kind.get_description(),
            RuleKind::InvalidTag(tag, invalid_tag_kind) => {
                format!("[InvalidTag `{}`]: ", tag) + &invalid_tag_kind.get_description()
            }
            RuleKind::InvalidKey(key_kind) => "[InvalidKey]: ".to_owned() + &key_kind.get_description(),
            RuleKind::InvalidEnumerator { identifier, kind } => {
                format!("[InvalidEnumerator `{}`]: ", identifier) + &kind.get_description()
            }
            RuleKind::InvalidEnum(id, kind) => format!("[InvalidEnum `{}`]: ", id) + &kind.get_description(),
            RuleKind::InvalidEncoding(kind) => "[InvalidEncoding]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidIdentifier(kind) => "[InvalidIdentifier]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidException(kind) => "[InvalidException]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidType(kind) => "[InvalidType]: ".to_owned() + &kind.get_description(),
            RuleKind::InvalidParameter(kind) => "[InvalidParameter]: ".to_owned() + &kind.get_description(),
            _ => "Todo".to_string(),
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

impl From<InvalidAttributeKind> for RuleKind {
    fn from(original: InvalidAttributeKind) -> RuleKind {
        RuleKind::InvalidAttribute(original)
    }
}

impl InvalidAttributeKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidAttributeKind::CompressAttributeCannotBeApplied => 0,
            InvalidAttributeKind::DeprecatedAttributeCannotBeApplied(_) => 5,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidAttributeKind::CompressAttributeCannotBeApplied => {
                "the compress attribute can only be applied to interfaces and operations".to_string()
            }
            InvalidAttributeKind::DeprecatedAttributeCannotBeApplied(kind) => {
                format!("the deprecated attribute cannot be applied to {}", kind)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidArgumentKind {
    ArgumentCannotBeEmpty(&'static str),
    ArgumentNotSupported(String, &'static str),
}

impl From<InvalidArgumentKind> for RuleKind {
    fn from(original: InvalidArgumentKind) -> RuleKind {
        RuleKind::InvalidArgument(original)
    }
}

impl InvalidArgumentKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidArgumentKind::ArgumentCannotBeEmpty(_) => 10,
            InvalidArgumentKind::ArgumentNotSupported(_, _) => 15,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidArgumentKind::ArgumentCannotBeEmpty(method) => format!("{} arguments cannot be empty", method),
            InvalidArgumentKind::ArgumentNotSupported(arg, method) => {
                format!("argument '{}' is not supported for `{}`", arg, method)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidKeyKind {
    CannotUseOptionalAsKey,
    StructsMustBeCompactToBeAKey,
    TypeCannotBeUsedAsAKey(String),
    StructContainsDisallowedType(String),
}

impl From<InvalidKeyKind> for RuleKind {
    fn from(original: InvalidKeyKind) -> RuleKind {
        RuleKind::InvalidKey(original)
    }
}

impl InvalidKeyKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidKeyKind::CannotUseOptionalAsKey => 20,
            InvalidKeyKind::StructsMustBeCompactToBeAKey => 25,
            InvalidKeyKind::TypeCannotBeUsedAsAKey(_) => 30,
            InvalidKeyKind::StructContainsDisallowedType(_) => 35,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidKeyKind::CannotUseOptionalAsKey => {
                "optional types cannot be used as a dictionary key type".to_string()
            }
            InvalidKeyKind::StructsMustBeCompactToBeAKey => {
                "structs must be compact to be used as a dictionary key type".to_string()
            }
            InvalidKeyKind::TypeCannotBeUsedAsAKey(identifier) => {
                format!("'{}' cannot be used as a dictionary key type", identifier)
            }
            InvalidKeyKind::StructContainsDisallowedType(identifier) => {
                format!(
                    "struct '{}' contains members that cannot be used as a dictionary key type",
                    identifier
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidEnumKind {
    UnderlyingTypeMustBeIntegral(String),
    CannotHaveOptionalUnderlyingType,
    MustContainAtLeastOneValue,
}

impl InvalidEnumKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidEnumKind::UnderlyingTypeMustBeIntegral(_) => 50,
            InvalidEnumKind::CannotHaveOptionalUnderlyingType => 60,
            InvalidEnumKind::MustContainAtLeastOneValue => 75,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidEnumKind::UnderlyingTypeMustBeIntegral(underlying) => {
                format!("underlying type '{}' is not supported for enums", underlying)
            }
            InvalidEnumKind::CannotHaveOptionalUnderlyingType => {
                "enums cannot have optional underlying types".to_string()
            }
            InvalidEnumKind::MustContainAtLeastOneValue => "enums must contain at least one enumerator".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidEnumeratorKind {
    MustBePositive,
    MustBeBounded { value: i64, min: i64, max: i64 },
    MustBeUnique,
}

impl InvalidEnumeratorKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidEnumeratorKind::MustBePositive => 40,
            InvalidEnumeratorKind::MustBeBounded { .. } => 45,
            InvalidEnumeratorKind::MustBeUnique => 55,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidEnumeratorKind::MustBePositive => "enumerators must be non-negative".to_owned(),
            InvalidEnumeratorKind::MustBeBounded { value, min, max } => format!(
                "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
                value = value,
                min = min,
                max = max,
            ),
            InvalidEnumeratorKind::MustBeUnique => "enumerators must be unique".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidIdentifierKind {
    IdentifierCannotBeARedefinition(String),
    IdentifierCannotShadowAnotherSymbol(String),
}
impl From<InvalidIdentifierKind> for RuleKind {
    fn from(original: InvalidIdentifierKind) -> RuleKind {
        RuleKind::InvalidIdentifier(original)
    }
}

impl InvalidIdentifierKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidIdentifierKind::IdentifierCannotBeARedefinition(_) => 80,
            InvalidIdentifierKind::IdentifierCannotShadowAnotherSymbol(_) => 85,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidIdentifierKind::IdentifierCannotBeARedefinition(identifier) => {
                format!("redefinition of `{}`", identifier)
            }
            InvalidIdentifierKind::IdentifierCannotShadowAnotherSymbol(identifier) => {
                format!("`{}` shadows another symbol", identifier)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidTagKind {
    TagsMustBeUnique,
}

impl InvalidTagKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidTagKind::TagsMustBeUnique => 1,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidTagKind::TagsMustBeUnique => "tags must be unique".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidParameterKind {
    RequiredParametersMustBeFirst,
    StreamsMustBeLast,
    ReturnTuplesMustContainAtleastTwoElements,
}

impl From<InvalidParameterKind> for RuleKind {
    fn from(original: InvalidParameterKind) -> RuleKind {
        RuleKind::InvalidParameter(original)
    }
}

impl InvalidParameterKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidParameterKind::RequiredParametersMustBeFirst => 1,
            InvalidParameterKind::StreamsMustBeLast => 2,
            InvalidParameterKind::ReturnTuplesMustContainAtleastTwoElements => 3,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidParameterKind::RequiredParametersMustBeFirst => {
                "required parameters must precede tagged parameters".to_string()
            }
            InvalidParameterKind::StreamsMustBeLast => {
                "only the last parameter in an operation can use the stream modifier".to_string()
            }
            InvalidParameterKind::ReturnTuplesMustContainAtleastTwoElements => {
                "return tuples must have at least 2 elements".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidMemberKind {
    TaggedDataMemberNotSupportedInCompactStructs,
    TaggedDataMemberMustBeOptional,
    TaggedDataMemberCannotBeClass,
}

impl InvalidMemberKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidMemberKind::TaggedDataMemberNotSupportedInCompactStructs => 90,
            InvalidMemberKind::TaggedDataMemberMustBeOptional => 95,
            InvalidMemberKind::TaggedDataMemberCannotBeClass => 100,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidMemberKind::TaggedDataMemberNotSupportedInCompactStructs => {
                "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact".to_string()
            }
            InvalidMemberKind::TaggedDataMemberMustBeOptional => "tagged members must be optional".to_string(),
            InvalidMemberKind::TaggedDataMemberCannotBeClass => "tagged members cannot be classes".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidExceptionKind {
    CanOnlyInheritFromSingleBase,
}
impl From<InvalidExceptionKind> for RuleKind {
    fn from(original: InvalidExceptionKind) -> RuleKind {
        RuleKind::InvalidException(original)
    }
}

impl InvalidExceptionKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidExceptionKind::CanOnlyInheritFromSingleBase => 1,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidExceptionKind::CanOnlyInheritFromSingleBase => {
                "exceptions can only inherit from a single base exception".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidTypeKind {
    TypeMismatch(String, String),
    ConcreteTypeMismatch(String, String),
}
impl From<InvalidTypeKind> for RuleKind {
    fn from(original: InvalidTypeKind) -> RuleKind {
        RuleKind::InvalidType(original)
    }
}

impl InvalidTypeKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidTypeKind::TypeMismatch(_, _) => 1,
            InvalidTypeKind::ConcreteTypeMismatch(_, _) => 2,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidTypeKind::TypeMismatch(expected, found) => {
                format!(
                    "type mismatch: expected a `{}` but found {} (which doesn't implement `{}`)",
                    expected, found, expected
                )
            }
            InvalidTypeKind::ConcreteTypeMismatch(expected, found) => {
                format!("type mismatch: expected `{}` but found `{}`", expected, found)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidStructKind {
    CompactStructIsEmpty,
}

impl InvalidStructKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidStructKind::CompactStructIsEmpty => 1,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidStructKind::CompactStructIsEmpty => "compact structs must be non-empty".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidEncodingKind {
    NotSupported {
        kind: String,
        identifier: String,
        encoding: String,
    },
    UnsupportedType {
        type_string: String,
        encoding: String,
    },
    ExceptionNotSupported(String),
    OptionalsNotSupported(String),
    StreamedParametersNotSupported(String),
}

impl From<InvalidEncodingKind> for RuleKind {
    fn from(original: InvalidEncodingKind) -> RuleKind {
        RuleKind::InvalidEncoding(original)
    }
}

impl InvalidEncodingKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidEncodingKind::NotSupported { .. } => 105,
            InvalidEncodingKind::UnsupportedType { .. } => 110,
            InvalidEncodingKind::ExceptionNotSupported { .. } => 115,
            InvalidEncodingKind::OptionalsNotSupported { .. } => 120,
            InvalidEncodingKind::StreamedParametersNotSupported { .. } => 125,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidEncodingKind::NotSupported {
                kind,
                identifier,
                encoding,
            } => {
                format!(
                    "{} `{}` is not supported by the Slice{} encoding",
                    kind, identifier, encoding,
                )
            }
            InvalidEncodingKind::OptionalsNotSupported(encoding) => {
                format!(
                    "optional types are not supported by the {} encoding (except for classes, proxies, and with tags)",
                    encoding
                )
            }
            InvalidEncodingKind::UnsupportedType { type_string, encoding } => {
                format!(
                    "the type `{}` is not supported by the Slice{} encoding",
                    type_string, encoding,
                )
            }
            InvalidEncodingKind::ExceptionNotSupported(encoding) => format!(
                "exceptions cannot be used as a data type with the Slice{} encoding",
                encoding
            ),
            InvalidEncodingKind::StreamedParametersNotSupported(encoding) => {
                format!("streamed parameters are not supported by the {} encoding", encoding)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidTypeAliasKind {
    SelfReferentialTypeAliasNeedsConcreteType(String),
}

impl InvalidTypeAliasKind {
    pub fn error_code(&self) -> u32 {
        match self {
            InvalidTypeAliasKind::SelfReferentialTypeAliasNeedsConcreteType(_) => 130,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidTypeAliasKind::SelfReferentialTypeAliasNeedsConcreteType(identifier) => {
                format!("self-referential type alias '{}' has no concrete type", identifier)
            }
        }
    }
}
