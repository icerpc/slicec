// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;

#[derive(Debug, Clone)]
pub enum RuleKind {
    InvalidAttribute(InvalidAttributeKind),
    InvalidArgument(InvalidArgumentKind),
    InvalidTag(String, InvalidTagKind),
    InvalidParameter(String, InvalidParameterKind),
    InvalidReturn,
    InvalidMember(String, InvalidMemberKind),
    InvalidType,
    InvalidEnum,
    InvalidEnumerator {
        identifier: String,
        kind: InvalidEnumeratorKind,
    },
    InvalidEncoding(InvalidEncodingKind),
    InvalidStruct(String, InvalidStructKind),
    InvalidIdentifier(InvalidIdentifierKind),
    InvalidException,
    InvalidModule,
    InvalidTypeAlias(InvalidTypeAliasKind),
    InvalidKey(InvalidKeyKind),
}

impl RuleKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            RuleKind::InvalidAttribute(invalid_attribute_kind) => invalid_attribute_kind.get_error_code(),
            _ => 0,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            RuleKind::InvalidAttribute(attribute_kind) => {
                "invalid attribute: ".to_owned() + &attribute_kind.get_description()
            }
            RuleKind::InvalidArgument(arg_kind) => "invalid argument: ".to_owned() + &arg_kind.get_description(),
            RuleKind::InvalidTag(tag, invalid_tag_kind) => {
                format!("invalid tag `{}`: ", tag) + &invalid_tag_kind.get_description()
            }
            _ => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidAttributeKind {
    CompressAttributeCannotBeApplied(),
    DeprecatedAttributeCannotBeApplied(String),
}

impl InvalidAttributeKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidAttributeKind::CompressAttributeCannotBeApplied() => 1,
            InvalidAttributeKind::DeprecatedAttributeCannotBeApplied(_) => 2,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidAttributeKind::CompressAttributeCannotBeApplied() => {
                "the compress attribute can only be applied to interfaces and operations".to_string()
            }
            InvalidAttributeKind::DeprecatedAttributeCannotBeApplied(kind) => {
                format!("the deprecated attribute cannot be applied to {}s", kind)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidArgumentKind {
    ArgumentCannotBeEmpty(String),
    ArgumentNotSupported(String, String),
}

impl InvalidArgumentKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidArgumentKind::ArgumentCannotBeEmpty(_) => 6,
            InvalidArgumentKind::ArgumentNotSupported(_, _) => 7,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidArgumentKind::ArgumentCannotBeEmpty(method) => format!("{} arguments cannot be empty", method),
            InvalidArgumentKind::ArgumentNotSupported(arg, method) => {
                format!("argument '{}' is not supported for {}", arg, method)
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

impl InvalidKeyKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidKeyKind::CannotUseOptionalAsKey => 0,
            InvalidKeyKind::StructsMustBeCompactToBeAKey => 1,
            InvalidKeyKind::TypeCannotBeUsedAsAKey(_) => 2,
            InvalidKeyKind::StructContainsDisallowedType(_) => 3,
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
pub enum InvalidEnumeratorKind {
    MustBeNonNegative,
    MustBeBounded { value: i64, min: i64, max: i64 },
    UnderlyingTypeMustBeIntegral(String),
    MustBeUnique,
    CannotHaveOptionalUnderlyingType,
    MustContainAtLeastOneValue,
}

impl InvalidEnumeratorKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidEnumeratorKind::MustBeNonNegative => 1,
            InvalidEnumeratorKind::MustBeBounded { .. } => 2,
            InvalidEnumeratorKind::UnderlyingTypeMustBeIntegral(_) => 3,
            InvalidEnumeratorKind::MustBeUnique => 4,
            InvalidEnumeratorKind::CannotHaveOptionalUnderlyingType => 5,
            InvalidEnumeratorKind::MustContainAtLeastOneValue => 6,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidEnumeratorKind::MustBeNonNegative => "enumerators must be non-negative".to_owned(),
            InvalidEnumeratorKind::MustBeBounded { value, min, max } => format!(
                "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
                value = value,
                min = min,
                max = max,
            ),
            InvalidEnumeratorKind::UnderlyingTypeMustBeIntegral(underlying) => {
                format!("underlying type '{}' is not allowed for enums", underlying)
            }
            InvalidEnumeratorKind::MustBeUnique => "enumerators must be unique".to_string(),
            InvalidEnumeratorKind::CannotHaveOptionalUnderlyingType => {
                "enums cannot have optional underlying types".to_string()
            }
            InvalidEnumeratorKind::MustContainAtLeastOneValue => {
                "enums must contain at least one enumerator".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidIdentifierKind {
    IdentifierCannotBeARedefinition(String),
    IdentifierCannotShadowAnotherSymbol(String),
}

impl InvalidIdentifierKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidIdentifierKind::IdentifierCannotBeARedefinition(_) => 1,
            InvalidIdentifierKind::IdentifierCannotShadowAnotherSymbol(_) => 2,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            InvalidIdentifierKind::IdentifierCannotBeARedefinition(identifier) => {
                format!("redefinition of {}", identifier)
            }
            InvalidIdentifierKind::IdentifierCannotShadowAnotherSymbol(identifier) => {
                format!("{} shadows another symbol", identifier)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidTagKind {
    TagsMustBeUnique,
}

impl InvalidTagKind {
    pub fn get_error_code(&self) -> u32 {
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
}

impl InvalidParameterKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidParameterKind::RequiredParametersMustBeFirst => 1,
            InvalidParameterKind::StreamsMustBeLast => 2,
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
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidMemberKind::TaggedDataMemberNotSupportedInCompactStructs => 1,
            InvalidMemberKind::TaggedDataMemberMustBeOptional => 2,
            InvalidMemberKind::TaggedDataMemberCannotBeClass => 2,
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
pub enum InvalidStructKind {
    CompactStructIsEmpty,
}

impl InvalidStructKind {
    pub fn get_error_code(&self) -> u32 {
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

impl InvalidEncodingKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidEncodingKind::NotSupported { .. } => 1,
            InvalidEncodingKind::UnsupportedType { .. } => 2,
            InvalidEncodingKind::ExceptionNotSupported { .. } => 3,
            InvalidEncodingKind::OptionalsNotSupported { .. } => 4,
            InvalidEncodingKind::StreamedParametersNotSupported { .. } => 5,
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
                "exceptions cannot be used as a data type with the {} encoding",
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
    pub fn get_error_code(&self) -> u32 {
        match self {
            InvalidTypeAliasKind::SelfReferentialTypeAliasNeedsConcreteType(_) => 1,
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
