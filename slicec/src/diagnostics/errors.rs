// Copyright (c) ZeroC, Inc.

use crate::utils::string_util::indefinite_article;

use std::ops::Range;

#[derive(Debug)]
pub enum Error {
    // ----------------  Generic Errors ---------------- //
    IO {
        action: &'static str,
        path: String,
        error: std::io::Error,
    },

    Syntax {
        message: String,
    },

    // ---------------- Dictionary Errors ---------------- //
    /// Dictionaries cannot use optional types as keys.
    KeyMustBeNonOptional,

    /// An unsupported type was used as a dictionary key type.
    KeyTypeNotSupported {
        /// The type and/or identifier of the type that was used as a dictionary key type.
        kind: String,
    },

    /// Struct contains a field that cannot be used as a dictionary key type.
    StructKeyContainsDisallowedType {
        /// The identifier of the struct.
        struct_identifier: String,
    },

    /// Structs must be compact to be used as a dictionary key type.
    StructKeyMustBeCompact,

    // ----------------  Enum Errors ---------------- //
    /// Enumerator values must be unique.
    DuplicateEnumeratorValue {
        /// The value of the enumerator that was already used.
        enumerator_value: i128,
    },

    /// Enumerators cannot contain fields when their enclosing enum has an underlying type.
    EnumeratorCannotContainFields {
        enumerator_identifier: String,
    },

    /// Enums cannot have optional underlying types.
    CannotUseOptionalUnderlyingType {
        /// The identifier of the enum.
        enum_identifier: String,
    },

    /// A type was marked 'compact' when it was invalid to do so.
    CannotBeCompact {
        /// The kind of type that was marked compact.
        kind: &'static str,
        /// The identifier of the type.
        identifier: String,
    },

    /// An enumerator was found that was out of bounds of the underlying type of the parent enum.
    EnumeratorValueOutOfBounds {
        /// The identifier of the enumerator.
        enumerator_identifier: String,
        /// The value of the out of bounds enumerator.
        value: i128,
        /// The minimum value of the underlying type of the enum.
        min: i128,
        /// The maximum value of the underlying type of the enum.
        max: i128,
    },

    /// Enums must be contain at least one enumerator.
    MustContainEnumerators {
        /// The identifier of the enum.
        enum_identifier: String,
    },

    /// Enum underlying types must be integral types.
    EnumUnderlyingTypeNotSupported {
        /// The identifier of the enum.
        enum_identifier: String,
        /// The name of the non-integral type that was used as the underlying type of the enum.
        kind: Option<String>,
    },

    // ----------------  Operation Errors ---------------- //
    /// A streamed parameter was not the last parameter in the operation.
    StreamedMembersMustBeLast {
        /// The identifier of the parameter that caused the error.
        parameter_identifier: String,
    },

    /// Return tuples for an operation must contain at least two element.
    ReturnTuplesMustContainAtLeastTwoElements,

    /// Multiple streamed parameters were used as parameters for an operation.
    MultipleStreamedMembers,

    // ----------------  Struct Errors ---------------- //
    /// Compact structs cannot be empty.
    CompactStructCannotBeEmpty,

    // ----------------  Tag Errors ---------------- //
    /// A duplicate tag value was found.
    CannotHaveDuplicateTag {
        /// The identifier of the tagged member.
        identifier: String,
    },

    /// A tag value was not in the expected range, 0 .. i32::MAX.
    TagValueOutOfBounds,

    /// A tagged member was not set to optional.
    TaggedMemberMustBeOptional {
        /// The identifier of the tagged member.
        identifier: String,
    },

    /// Compact types cannot contain tagged fields.
    CompactTypeCannotContainTaggedFields {
        /// The kind of type that contains the fields.
        kind: &'static str,
    },

    // ----------------  General Errors ---------------- //
    /// An identifier was redefined.
    Redefinition {
        /// The identifier that was redefined.
        identifier: String,
    },

    /// A self-referential type alias has no concrete type.
    SelfReferentialTypeAliasNeedsConcreteType {
        /// The name of the type alias.
        identifier: String,
    },

    /// An identifier was used to shadow another identifier.
    Shadows {
        /// The identifier that is shadowing a previously defined identifier.
        identifier: String,
    },

    /// Used to indicate when two types should match, but do not.
    TypeMismatch {
        /// The name of the expected kind.
        expected: String,
        /// The name of the found kind.
        actual: String,
        /// Whether the expected type was a concrete type (true) or a trait type (false).
        is_concrete: bool,
    },

    /// An integer literal was outside the parsable range of 0..i128::MAX.
    IntegerLiteralOverflows,

    /// An integer literal contained illegal characters for its base.
    InvalidIntegerLiteral {
        /// The base of the integer literal; Ex: 16 (hex), 10 (dec).
        base: u32,
    },

    /// An self-referential type had an infinite size cycle.
    InfiniteSizeCycle {
        /// The type id of the type that caused the error.
        type_id: String,
        /// The cycle that was found.
        cycle: String,
    },

    /// No element with the specified identifier was found.
    DoesNotExist {
        /// The identifier that was not found.
        identifier: String,
    },

    // ----------------  Attribute Errors ---------------- //
    /// An attribute was applied to a Slice element for which it's invalid.
    /// For example: applying `[oneway]` to a struct ('oneway' is only allowed on operations).
    InvalidAttribute {
        /// the directive of the invalid attribute.
        directive: String,
    },

    /// An unknown attribute was encountered, which uses a known prefix.
    /// For example: if a C# code-generator encountered the following attribute: `[cs::foobar]`.
    UnknownAttribute {
        /// The directive of the unknown attribute.
        directive: String,
    },

    /// An element requires a specific attribute to be applied to it, which is not present.
    /// For example: custom types must have their mapped type specified with a `[xxx:type(...)]` attribute.
    MissingRequiredAttribute {
        /// The missing attribute; should include placeholder names for expected arguments.
        expected_attribute: String,
    },

    /// A non-repeatable attribute was applied to the same element multiple times.
    /// For example: `[compress(Args)] [compress(Return)] myOperation()`; 'compress' is not repeatable.
    AttributeIsNotRepeatable {
        /// The directive of the non-repeatable attribute.
        directive: String,
    },

    /// An invalid argument was provided to an attribute which otherwise accepts arguments.
    /// For example: `[compress(FooBar)] myOperation()`; 'compress' accepts arguments but 'FooBar' is not a valid one.
    InvalidAttributeArgument {
        /// The directive of the attribute.
        directive: String,
        /// the invalid argument that was provided.
        argument: String,
    },

    /// Too few or too many arguments were supplied to an otherwise valid attribute.
    /// For example: `[oneway(FooBar)]` ('oneway' takes no arguments) or `[compress]` ('compress' requires arguments).
    IncorrectAttributeArgumentCount {
        /// The directive of the attribute.
        directive: String,
        /// A range representing the number of arguments this attribute can be given.
        expected_count: Range<usize>,
        /// The number of arguments this attribute was actually given.
        actual_count: usize,
    },

    // ----------------  Type Alias Errors ---------------- //
    /// A type alias had an optional underlying type.
    TypeAliasOfOptional,
}

impl Error {
    /// Returns the error code corresponding to this particular [`Error`].
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::IO { .. } => "E001",
            Self::Syntax { .. } => "E002",
            Self::KeyMustBeNonOptional => "E003",
            Self::StructKeyMustBeCompact => "E004",
            Self::KeyTypeNotSupported { .. } => "E005",
            Self::StructKeyContainsDisallowedType { .. } => "E006",
            Self::CannotUseOptionalUnderlyingType { .. } => "E007",
            Self::MustContainEnumerators { .. } => "E008",
            Self::EnumUnderlyingTypeNotSupported { .. } => "E009",
            Self::Redefinition { .. } => "E010",
            Self::Shadows { .. } => "E011",
            Self::CannotHaveDuplicateTag { .. } => "E012",
            Self::StreamedMembersMustBeLast { .. } => "E013",
            Self::ReturnTuplesMustContainAtLeastTwoElements => "E014",
            Self::CompactTypeCannotContainTaggedFields { .. } => "E015",
            Self::TaggedMemberMustBeOptional { .. } => "E016",
            Self::TypeMismatch { .. } => "E017",
            Self::CompactStructCannotBeEmpty => "E018",
            Self::SelfReferentialTypeAliasNeedsConcreteType { .. } => "E019",
            Self::EnumeratorValueOutOfBounds { .. } => "E020",
            Self::TagValueOutOfBounds => "E021",
            Self::DuplicateEnumeratorValue { .. } => "E022",
            Self::InvalidAttribute { .. } => "E023",
            Self::UnknownAttribute { .. } => "E024",
            Self::MissingRequiredAttribute { .. } => "E025",
            Self::AttributeIsNotRepeatable { .. } => "E026",
            Self::InvalidAttributeArgument { .. } => "E027",
            Self::IncorrectAttributeArgumentCount { .. } => "E028",
            Self::MultipleStreamedMembers => "E029",
            Self::IntegerLiteralOverflows => "E030",
            Self::InvalidIntegerLiteral { .. } => "E031",
            Self::InfiniteSizeCycle { .. } => "E032",
            Self::DoesNotExist { .. } => "E033",
            Self::TypeAliasOfOptional => "E034",
            Self::EnumeratorCannotContainFields { .. } => "E035",
            Self::CannotBeCompact { .. } => "E036",
        }
    }

    /// Returns a message describing this [`Error`] in detail.
    pub fn message(&self) -> String {
        match self {
            Self::IO { action, path, error } => {
                let message = match error.kind() {
                    std::io::ErrorKind::NotFound => "No such file or directory".to_owned(),
                    _ => error.to_string(),
                };
                format!("unable to {action} '{path}': {message}")
            }

            Self::Syntax { message } => format!("invalid syntax: {message}"),

            Self::KeyMustBeNonOptional => "optional types are not valid dictionary key types".to_owned(),

            Self::StructKeyMustBeCompact => "structs must be compact to be used as a dictionary key type".to_owned(),

            Self::KeyTypeNotSupported { kind } => format!("invalid dictionary key type: {kind}"),

            Self::StructKeyContainsDisallowedType { struct_identifier }
                => format!("struct '{struct_identifier}' contains fields that are not a valid dictionary key types"),

            Self::CannotUseOptionalUnderlyingType { enum_identifier }
                => format!("invalid enum '{enum_identifier}': enums cannot have optional underlying types"),

            Self::MustContainEnumerators { enum_identifier }
                => format!("invalid enum '{enum_identifier}': enums must contain at least one enumerator"),

            Self::EnumUnderlyingTypeNotSupported { enum_identifier, kind } => {
                if let Some(kind) = kind {
                    format!("invalid enum '{enum_identifier}': underlying type '{kind}' is not supported", )
                } else {
                    format!("invalid enum '{enum_identifier}': missing required underlying type")
                }
            }

            Self::Redefinition { identifier } => format!("redefinition of '{identifier}'"),

            Self::Shadows { identifier } => format!("'{identifier}' shadows another symbol"),

            Self::CannotHaveDuplicateTag { identifier }
                => format!("invalid tag on member '{identifier}': tags must be unique"),

            Self::StreamedMembersMustBeLast { parameter_identifier }
                => format!("invalid parameter '{parameter_identifier}': only the last parameter in an operation can use the stream modifier"),

            Self::ReturnTuplesMustContainAtLeastTwoElements => "return tuples must have at least 2 elements".to_owned(),

            Self::CompactTypeCannotContainTaggedFields { kind }
                => format!("tagged fields are not supported in compact {kind}s; consider removing the tag, or making the {kind} non-compact"),

            Self::TaggedMemberMustBeOptional { identifier }
                => format!("invalid tag on member '{identifier}': tagged members must be optional"),

            Self::TypeMismatch { expected, actual, is_concrete } => {
                format!(
                    "type mismatch: expected {} '{expected}' but found {} '{actual}'{}",
                    indefinite_article(expected),
                    indefinite_article(actual),
                    if *is_concrete {
                        "".to_owned()
                    } else {
                        format!(" (which isn't {} '{expected}')", indefinite_article(expected))
                    }
                )
            }

            Self::CompactStructCannotBeEmpty => "compact structs must be non-empty".to_owned(),

            Self::SelfReferentialTypeAliasNeedsConcreteType { identifier }
                => format!("self-referential type alias '{identifier}' has no concrete type"),

            Self::EnumeratorValueOutOfBounds { enumerator_identifier, value, min, max }
                => format!("invalid enumerator '{enumerator_identifier}': enumerator value '{value}' is out of bounds. The value must be between '{min}..{max}', inclusive"),

            Self::TagValueOutOfBounds => "tag values must be within the range 0 <= value <= 2147483647".to_owned(),

            Self::DuplicateEnumeratorValue { enumerator_value }
                => format!("enumerator values must be unique; the value '{enumerator_value}' is already in use"),

            Self::InvalidAttribute { directive } => format!("invalid attribute '{directive}'"),

            Self::UnknownAttribute { directive } => format!("unknown attribute '{directive}'"),

            Self::MissingRequiredAttribute { expected_attribute }
                => format!("missing required attribute '{expected_attribute}'"),

            Self::AttributeIsNotRepeatable { directive } => format!("duplicate attribute '{directive}'"),

            Self::InvalidAttributeArgument { directive, argument }
                => format!("'{argument}' is not a valid argument to the '{directive}' attribute"),

            Self::IncorrectAttributeArgumentCount { directive, expected_count, actual_count } => {
                let args_provided = if *actual_count == 1 {
                    String::from("1 argument was provided")
                } else {
                    format!("{actual_count} arguments were provided")
                };
                if expected_count.len() == 1 {
                    // If the range is only 1 element long, then this attribute requires an exact number of arguments.
                    let expected = expected_count.start;
                    if expected == 0 {
                        format!("attribute '{directive}' does not take any arguments, but {args_provided}")
                    } else {
                        let only = if expected > *actual_count { "only " } else { "" };
                        let args = if expected == 1 { "argument" } else { "arguments" };
                        format!("attribute '{directive}' takes exactly {expected} {args}, but {only}{args_provided}")
                    }
                } else if expected_count.end == usize::MAX {
                    // If the range has no upper bound, then this attribute only requires a minimum number of arguments.
                    let expected = expected_count.start;
                    let args = if expected == 1 { "argument" } else { "arguments" };
                    format!("attribute '{directive}' requires at least {expected} {args}, but only {args_provided}")
                } else {
                    // Otherwise, this attribute accepts a specific range of possible arguments.
                    let min = expected_count.start;
                    let max = expected_count.end - 1;
                    format!("attribute '{directive}' takes between {min} and {max} arguments (inclusive), but {args_provided}")
                }
            }

            Self::MultipleStreamedMembers => "cannot have multiple streamed members".to_owned(),

            Self::IntegerLiteralOverflows
                => "integer literal is outside the parsable range of -2^127 <= i <= 2^127 - 1".to_owned(),

            Self::InvalidIntegerLiteral { base } => format!("integer literal contains illegal characters for base-{base}"),

            Self::InfiniteSizeCycle { type_id, cycle } => format!("type {type_id} illegally references itself: {cycle}"),

            Self::DoesNotExist { identifier } => format!("no element with identifier '{identifier}' exists"),

            Self::TypeAliasOfOptional => "optional types cannot be aliased".to_owned(),

            Self::EnumeratorCannotContainFields { enumerator_identifier }
                => format!("invalid enumerator '{enumerator_identifier}': fields cannot be declared within enums that specify an underlying type"),

            Self::CannotBeCompact { kind, identifier } => format!("'{kind}' '{identifier}' cannot be marked compact"),
        }
    }
}
