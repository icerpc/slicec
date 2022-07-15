// // Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::ErrorKind;

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
    InvalidStruct(String, InvalidStructKind),
    InvalidIdentifier(InvalidIdentifierKind),
    InvalidException,
    InvalidModule,
    InvalidTypeAlias,
    InvalidKey(InvalidKeyKind),
}

impl ErrorKind for RuleKind {
    fn get_error_code(&self) -> u32 {
        match self {
            RuleKind::InvalidAttribute(invalid_attribute_kind) => invalid_attribute_kind.get_error_code(),
            _ => 0,
        }
    }

    fn get_description(&self) -> String {
        match self {
            RuleKind::InvalidAttribute(invalid_attribute_kind) => invalid_attribute_kind.get_description(),
            _ => "".to_string(),
        }
    }
}

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
