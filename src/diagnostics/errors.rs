// Copyright (c) ZeroC, Inc.

use crate::grammar::Encoding;
use crate::implement_diagnostic_functions;
use in_definite;

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

    // ----------------  Encoding Errors ---------------- //
    /// The user specified an encoding multiple times in a single Slice file.
    MultipleEncodingVersions,

    /// The provided kind with identifier is not supported in the specified encoding.
    NotSupportedWithEncoding {
        /// The kind that is not supported.
        kind: String,
        /// The identifier of the kind that is not supported.
        identifier: String,
        /// The encoding that was specified.
        encoding: Encoding,
    },

    /// Optional are not supported in the specified encoding.
    OptionalsNotSupported {
        /// The encoding that was specified.
        encoding: Encoding,
    },

    /// Streamed parameters are not supported with the specified encoding.
    StreamedParametersNotSupported {
        ///  The encoding that was specified.
        encoding: Encoding,
    },

    /// A non-Slice1 operation used the `AnyException` keyword.
    AnyExceptionNotSupported,

    /// An unsupported type was used in the specified encoding.
    UnsupportedType {
        /// The name of the kind that was used in the specified encoding.
        kind: String,
        /// The encoding that was specified.
        encoding: Encoding,
    },

    // ----------------  Enum Errors ---------------- //
    /// Enumerator values must be unique.
    DuplicateEnumeratorValue {
        /// The value of the enumerator that was already used.
        enumerator_value: i128,
    },

    /// Enums cannot have optional underlying types.
    CannotUseOptionalUnderlyingType {
        /// The identifier of the enum.
        enum_identifier: String,
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
    UnderlyingTypeMustBeIntegral {
        /// The identifier of the enum.
        enum_identifier: String,
        /// The name of the non-integral type that was used as the underlying type of the enum.
        kind: String,
    },

    // ----------------  Exception Errors ---------------- //
    /// Exceptions cannot be used as a data type with the specified encoding.
    ExceptionNotSupported {
        /// The encoding that was specified.
        encoding: Encoding,
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

    /// Compact structs cannot contain tagged fields.
    CompactStructCannotContainTaggedFields,

    // ----------------  Tag Errors ---------------- //
    /// A duplicate tag value was found.
    CannotHaveDuplicateTag {
        /// The identifier of the tagged member.
        identifier: String,
    },

    /// Cannot tag a class.
    CannotTagClass {
        /// The identifier of the tagged member.
        identifier: String,
    },

    /// Cannot tag a member that contains a class.
    CannotTagContainingClass {
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

    // ----------------  General Errors ---------------- //
    /// A compact ID was not in the expected range, 0 .. i32::MAX.
    CompactIdOutOfBounds,

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

    /// An invalid Slice encoding was used.
    InvalidEncodingVersion {
        /// The encoding version that was used.
        encoding: String,
    },

    /// A file scoped module contained submodules.
    FileScopedModuleCannotContainSubModules {
        identifier: String,
    },

    /// A malformed or invalid Warning code was supplied to the ignore warnings attribute.
    InvalidWarningCode {
        /// The invalid warning code.
        code: String,
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
    /// An invalid argument was provided to an attribute directive.
    ArgumentNotSupported {
        /// The argument that was provided.
        argument: String,
        /// The directive it was provided to.
        directive: String,
    },

    // The following are errors that are needed to report cs attribute errors.
    MissingRequiredArgument {
        argument: String,
    },

    MissingRequiredAttribute {
        attribute: String,
    },

    TooManyArguments {
        expected: String,
    },

    UnexpectedAttribute {
        attribute: String,
    },

    AttributeIsNotRepeatable {
        attribute: String,
    },

    // ----------------  Type Alias Errors ---------------- //
    /// A type alias had an optional underlying type.
    TypeAliasOfOptional,
}

implement_diagnostic_functions!(
    Error,
    (
        "E001",
        IO,
        format!("unable to {action} '{path}': {}", io_error_message(error)),
        action,
        path,
        error
    ),
    (
        "E002",
        Syntax,
        format!("{message}"),
        message
    ),
    (
        "E004",
        ArgumentNotSupported,
        format!("'{argument}' is not a legal argument for the '{directive}' attribute"),
        argument,
        directive
    ),
    (
        "E005",
        KeyMustBeNonOptional,
        "optional types are not valid dictionary key types"
    ),
    (
        "E006",
        StructKeyMustBeCompact,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        "E007",
        KeyTypeNotSupported,
        format!("invalid dictionary key type: {kind}"),
        kind
    ),
    (
        "E008",
        StructKeyContainsDisallowedType,
        format!("struct '{struct_identifier}' contains fields that are not a valid dictionary key types"),
        struct_identifier
    ),
    (
        "E009",
        CannotUseOptionalUnderlyingType,
        format!("invalid enum '{enum_identifier}': enums cannot have optional underlying types"),
        enum_identifier
    ),
    (
        "E010",
        MustContainEnumerators,
        format!("invalid enum '{enum_identifier}': enums must contain at least one enumerator"),
        enum_identifier
    ),
    (
        "E011",
        UnderlyingTypeMustBeIntegral,
        format!("invalid enum '{enum_identifier}': underlying type '{kind}' is not supported for enums"),
        enum_identifier,
        kind
    ),
    (
        "E012",
        Redefinition,
        format!("redefinition of '{identifier}'"),
        identifier
    ),
    (
        "E013",
        Shadows,
        format!("'{identifier}' shadows another symbol"),
        identifier
    ),
    (
        "E014",
        CannotHaveDuplicateTag,
        format!("invalid tag on member '{identifier}': tags must be unique"),
        identifier
    ),
    (
        "E016",
        StreamedMembersMustBeLast,
        format!("invalid parameter '{parameter_identifier}': only the last parameter in an operation can use the stream modifier"),
        parameter_identifier
    ),
    (
        "E017",
        ReturnTuplesMustContainAtLeastTwoElements,
        "return tuples must have at least 2 elements"
    ),
    (
        "E018",
        CompactStructCannotContainTaggedFields,
        "tagged fields are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        "E019",
        TaggedMemberMustBeOptional,
        format!("invalid tag on member '{identifier}': tagged members must be optional"),
        identifier
    ),
    (
        "E020",
        CannotTagClass,
        format!("invalid tag on member '{identifier}': tagged members cannot be classes"),
        identifier
    ),
    (
        "E021",
        CannotTagContainingClass,
        format!("invalid tag on member '{identifier}': tagged members cannot contain classes"),
        identifier
    ),
    (
        "E022",
        TypeMismatch,
        format!(
            "type mismatch: expected {} '{expected}' but found {} '{actual}'{}",
            in_definite::get_a_or_an(expected),
            in_definite::get_a_or_an(actual),
            if *is_concrete {
                "".to_owned()
            } else {
                format!(" (which isn't {} '{expected}')", in_definite::get_a_or_an(expected))
            }
        ),
        expected,
        actual,
        is_concrete
    ),
    (
        "E024",
        CompactStructCannotBeEmpty,
        "compact structs must be non-empty"
    ),
    (
        "E025",
        SelfReferentialTypeAliasNeedsConcreteType,
        format!("self-referential type alias '{identifier}' has no concrete type"),
        identifier
    ),
    (
        "E026",
        EnumeratorValueOutOfBounds,
        format!(
            "invalid enumerator '{enumerator_identifier}': enumerator value '{value}' is out of bounds. The value must be between '{min}..{max}', inclusive",
        ),
        enumerator_identifier, value, min, max
    ),
    (
        "E027",
        TagValueOutOfBounds,
        "tag values must be within the range 0 <= value <= 2147483647"
    ),
    (
        "E028",
        DuplicateEnumeratorValue,
        format!("enumerator values must be unique; the value '{enumerator_value}' is already in use"),
        enumerator_value
    ),
    (
        "E029",
        NotSupportedWithEncoding,
        format!("{kind} '{identifier}' is not supported by the {encoding} encoding"),
        kind, identifier, encoding
    ),
    (
        "E030",
        UnsupportedType,
        format!("the type '{kind}' is not supported by the {encoding} encoding"),
        kind,
        encoding
    ),
    (
        "E031",
        ExceptionNotSupported,
        format!("exceptions cannot be used as a data type with the {encoding} encoding"),
        encoding
    ),
    (
        "E032",
        OptionalsNotSupported,
        format!("optional types are not supported by the {encoding} encoding (except for classes, proxies, and with tags)"),
        encoding
    ),
    (
        "E033",
        StreamedParametersNotSupported,
        format!("streamed parameters are not supported by the {encoding} encoding"),
        encoding
    ),
    (
        "E034",
        UnexpectedAttribute,
        format!("unexpected attribute '{attribute}'"),
        attribute
    ),
    (
        "E035",
        MissingRequiredArgument,
        format!("missing required argument '{argument}'"),
        argument
    ),
    (
        "E036",
        TooManyArguments,
        format!("too many arguments, expected '{expected}'"),
        expected
    ),
    (
        "E037",
        MissingRequiredAttribute,
        format!("missing required attribute '{attribute}'"),
        attribute
    ),
    (
        "E038",
        MultipleStreamedMembers,
        "cannot have multiple streamed members"
    ),
    (
        "E039",
        CompactIdOutOfBounds,
        "compact IDs must be within the range 0 <= ID <= 2147483647"
    ),
    (
        "E040",
        IntegerLiteralOverflows,
        "integer literal is outside the parsable range of -2^127 <= i <= 2^127 - 1"
    ),
    (
        "E041",
        InvalidIntegerLiteral,
        format!("integer literal contains illegal characters for base-{base}"),
        base
    ),
    (
        "E042",
        InvalidEncodingVersion,
        format!("'{encoding}' is not a valid Slice encoding version"),
        encoding
    ),
    (
        "E043",
        MultipleEncodingVersions,
        "only a single encoding can be specified per file".to_owned()
    ),
    (
        "E044",
        FileScopedModuleCannotContainSubModules,
        format!("file scoped module '{identifier}' cannot contain sub modules"),
        identifier
    ),
    (
        "E045",
        AnyExceptionNotSupported,
        format!("operations that throw AnyException are only supported by the Slice1 encoding")

    ),
    (
        "E046",
        InvalidWarningCode,
        format!("the warning code '{code}' is not valid"),
        code
    ),
    (
        "E047",
        InfiniteSizeCycle,
        format!("self-referential type {type_id} has infinite size.\n{cycle}"),
        type_id, cycle
    ),
    (
        "E049",
        DoesNotExist,
        format!("no element with identifier '{identifier}' exists"),
        identifier
    ),
    (
        "E050",
        AttributeIsNotRepeatable,
        format!("duplicate attribute '{attribute}'"),
        attribute
    ),
    (
        "E051",
        TypeAliasOfOptional,
        "optional types cannot be aliased"
    )
);

fn io_error_message(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_owned(),
        _ => error.to_string(),
    }
}
