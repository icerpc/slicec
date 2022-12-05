// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::Encoding;
use crate::implement_error_functions;

#[derive(Debug)]
pub enum ErrorKind {
    // ----------------  Generic Errors ---------------- //
    IO(std::io::Error),

    Syntax(String),

    // ----------------  Attribute Errors ---------------- //
    /// Used to indicate when the compress attribute cannot be applied.
    CompressAttributeCannotBeApplied,

    /// Used to indicate when the deprecated attribute cannot be applied.
    ///
    /// # Fields
    ///
    /// * `kind` - The kind which the deprecated attribute was applied to.
    DeprecatedAttributeCannotBeApplied(String),

    // ----------------  Argument Errors ---------------- //
    /// The provided argument is not supported for the given method.
    ///
    /// # Fields
    ///
    /// * `argument_name` - The name of the argument.
    /// * `method_name` - The name of the method.
    ArgumentNotSupported(String, String),

    // ---------------- Dictionary Errors ---------------- //
    /// Dictionaries cannot use optional types as keys.
    KeyMustBeNonOptional,

    /// An unsupported type was used as a dictionary key type.
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier of the type that was used as a dictionary key type.
    KeyTypeNotSupported(String),

    /// Struct contains a member that cannot be used as a dictionary key type.
    ///
    /// # Fields
    ///
    /// * `struct_identifier` - The identifier of the struct.
    StructKeyContainsDisallowedType(String),

    /// Structs must be compact to be used as a dictionary key type.
    StructKeyMustBeCompact,

    // ----------------  Encoding Errors ---------------- //
    /// The user specified an encoding multiple times in a single Slice file.
    MultipleEncodingVersions,

    /// The provided kind with identifier is not supported in the specified encoding.
    ///
    /// # Fields
    ///
    /// * `kind` - The kind that was is not supported.
    /// * `identifier` - The identifier of the kind that is not supported.
    /// * `encoding` - The encoding that was specified.
    NotSupportedWithEncoding(String, String, Encoding),

    /// Optional are not supported in the specified encoding.
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified.
    OptionalsNotSupported(Encoding),

    /// Streamed parameters are not supported with the specified encoding.
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified.
    StreamedParametersNotSupported(Encoding),

    /// A non-Slice1 operation used the `AnyException` keyword.
    AnyExceptionNotSupported,

    /// An unsupported type was used in the specified encoding.
    ///
    /// # Fields
    ///
    /// * `kind` - The name of the kind that was used in the specified encoding.
    /// * `encoding` - The encoding that was specified.
    UnsupportedType(String, Encoding),

    // ----------------  Enum Errors ---------------- //
    /// Enumerator values must be unique.
    ///
    /// # Fields
    ///
    /// * `enumerator_value` - The value of the enumerator that was already used.
    DuplicateEnumeratorValue(i128),

    /// Enums cannot have optional underlying types.
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum.
    CannotUseOptionalUnderlyingType(String),

    /// An enumerator was found that was out of bounds of the underlying type of the parent enum.
    ///
    /// # Fields
    ///
    /// * `enumerator_identifier` - The identifier of the enumerator.
    /// * `value` - The value of the out of bounds enumerator.
    /// * `min` - The minimum value of the underlying type of the enum.
    /// * `max` - The maximum value of the underlying type of the enum.
    EnumeratorValueOutOfBounds(String, i128, i128, i128),

    /// Enums must be contain at least one enumerator.
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum.
    MustContainEnumerators(String),

    /// Enum underlying types must be integral types.
    ///
    /// # Fields
    ///
    /// * `enum_identifier` - The identifier of the enum.
    /// * `kind` - The name of the non-integral type that was used as the underlying type of the enum.
    UnderlyingTypeMustBeIntegral(String, String),

    // ----------------  Exception Errors ---------------- //
    /// Exceptions cannot be used as a data type with the specified encoding.
    ///
    /// # Fields
    ///
    /// * `encoding` - The encoding that was specified.
    ExceptionNotSupported(Encoding),

    // ----------------  Operation Errors ---------------- //
    /// A streamed parameter was not the last parameter in the operation.
    ///
    /// # Fields
    ///
    /// * `parameter_identifier` - The identifier of the parameter that caused the error.
    StreamedMembersMustBeLast(String),

    /// The required parameters of an operation did not precede the optional parameters.
    ///
    /// # Fields
    ///
    /// * `parameter_identifier` - The identifier of the parameter that caused the error.
    RequiredMustPrecedeOptional(String),

    /// Return tuples for an operation must contain at least two element.
    ReturnTuplesMustContainAtLeastTwoElements,

    /// Multiple streamed parameters were used as parameters for an operation.
    MultipleStreamedMembers,

    // ----------------  Struct Errors ---------------- //
    /// Compact structs cannot be empty.
    CompactStructCannotBeEmpty,

    /// Compact structs cannot contain tagged data members.
    CompactStructCannotContainTaggedMembers,

    // ----------------  Tag Errors ---------------- //
    /// A duplicate tag value was found.
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member.
    CannotHaveDuplicateTag(String),

    /// Cannot tag a class.
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member.
    CannotTagClass(String),

    /// Cannot tag a member that contains a class.
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member.
    CannotTagContainingClass(String),

    /// A tag value was not in the expected range, 0 .. i32::MAX.
    TagValueOutOfBounds,

    /// A tagged data member was not set to optional.
    ///
    /// # Fields
    ///
    /// * `member_identifier` - The identifier of the tagged member.
    TaggedMemberMustBeOptional(String),

    // ----------------  General Errors ---------------- //
    /// A compact ID was not in the expected range, 0 .. i32::MAX.
    CompactIdOutOfBounds,

    /// Used to indicate when a method must contain arguments.
    ///
    /// # Fields
    ///
    /// * `method_name` - The name of the method.
    CannotBeEmpty(String),

    /// Used to indicate when two concrete types should match, but do not.
    ///
    /// # Fields
    ///
    /// * `expected kind` - The name of the expected kind.
    /// * `actual kind` - The name of the found kind.
    ConcreteTypeMismatch(String, String),

    /// An identifier was redefined.
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier that was redefined.
    Redefinition(String),

    /// A self-referential type alias has no concrete type.
    ///
    /// # Fields
    ///
    /// * `identifier` - The name of the type alias.
    SelfReferentialTypeAliasNeedsConcreteType(String),

    /// An identifier was used to shadow another identifier.
    ///
    /// # Fields
    ///
    /// * `identifier` - The identifier that is shadowing previously defined identifier.
    Shadows(String),

    /// Used to indicate when two types should match, but do not.
    ///
    /// # Fields
    ///
    /// * `expected kind` - The name of the expected kind.
    /// * `actual kind` - The name of the found kind.
    TypeMismatch(String, String),

    /// An integer literal was outside the parsable range of 0..i128::MAX.
    IntegerLiteralOverflows,

    /// An integer literal contained illegal characters for its base.
    ///
    /// # Fields
    ///
    /// * `base` - The base of the integer literal; Ex: 16 (hex), 10 (dec).
    InvalidIntegerLiteral(u32),

    /// An invalid Slice encoding was used.
    InvalidEncodingVersion(i128),

    /// A file scoped module contained submodules.
    FileScopedModuleCannotContainSubModules(String),

    /// A malformed or invalid Warning code was supplied to the ignore warnings attribute.
    ///
    /// # Fields
    InvalidWarningCode(String),

    // ----------------  Attribute Errors ---------------- //
    // The following are errors that are needed to report cs attribute errors.
    MissingRequiredArgument(String), // (arg)

    MissingRequiredAttribute(String), // (attribute)

    TooManyArguments(String), // (expected)

    UnexpectedAttribute(String), // (attribute)

    InvalidAttribute(String, String), // (attribute, kind)
}

implement_error_functions!(
    ErrorKind,
    (
        ErrorKind::IO,
        format!("{io_error}"),
        io_error
    ),
    (
        ErrorKind::Syntax,
        format!("{message}"),
        message
    ),
    (
        "E001",
        ErrorKind::CompressAttributeCannotBeApplied,
        "the compress attribute can only be applied to interfaces and operations"
    ),
    (
        "E002",
        ErrorKind::DeprecatedAttributeCannotBeApplied,
        format!("the deprecated attribute cannot be applied to {kind}"),
        kind
    ),
    (
        "E003",
        ErrorKind::CannotBeEmpty,
        format!("{method} arguments cannot be empty"),
        method
    ),
    (
        "E004",
        ErrorKind::ArgumentNotSupported,
        format!("argument '{arg}' is not supported for `{method}`"),
        arg,
        method
    ),
    (
        "E005",
        ErrorKind::KeyMustBeNonOptional,
        "optional types cannot be used as a dictionary key type"
    ),
    (
        "E006",
        ErrorKind::StructKeyMustBeCompact,
        "structs must be compact to be used as a dictionary key type"
    ),
    (
        "E007",
        ErrorKind::KeyTypeNotSupported,
        format!("'{identifier}' cannot be used as a dictionary key type"),
        identifier
    ),
    (
        "E008",
        ErrorKind::StructKeyContainsDisallowedType,
        format!("struct '{identifier}' contains members that cannot be used as a dictionary key type"),
        identifier
    ),
    (
        "E009",
        ErrorKind::CannotUseOptionalUnderlyingType,
        format!("invalid enum `{}`: enums cannot have optional underlying types", identifier),
        identifier
    ),
    (
        "E010",
        ErrorKind::MustContainEnumerators,
        format!("invalid enum `{}`: enums must contain at least one enumerator", identifier),
        identifier
    ),
    (
        "E011",
        ErrorKind::UnderlyingTypeMustBeIntegral,
        format!("invalid enum `{identifier}`: underlying type '{underlying}' is not supported for enums"),
        identifier,
        underlying
    ),
    (
        "E012",
        ErrorKind::Redefinition,
        format!("redefinition of `{identifier}`"),
        identifier
    ),
    (
        "E013",
        ErrorKind::Shadows,
        format!("`{identifier}` shadows another symbol"),
        identifier
    ),
    (
        "E014",
        ErrorKind::CannotHaveDuplicateTag,
        format!("invalid tag on member `{}`: tags must be unique", identifier),
        identifier
    ),
    (
        "E016",
        ErrorKind::RequiredMustPrecedeOptional,
        format!("invalid parameter `{}`: required parameters must precede tagged parameters", identifier),
        identifier
    ),
    (
        "E017",
        ErrorKind::StreamedMembersMustBeLast,
        format!("invalid parameter `{}`: only the last parameter in an operation can use the stream modifier", identifier),
        identifier
    ),
    (
        "E018",
        ErrorKind::ReturnTuplesMustContainAtLeastTwoElements,
        "return tuples must have at least 2 elements"
    ),
    (
        "E019",
        ErrorKind::CompactStructCannotContainTaggedMembers,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        "E020",
        ErrorKind::TaggedMemberMustBeOptional,
        format!("invalid tag on member `{}`: tagged members must be optional", identifier),
        identifier
    ),
    (
        "E021",
        ErrorKind::CannotTagClass,
        format!("invalid tag on member `{}`: tagged members cannot be classes", identifier),
        identifier
    ),
    (
        "E022",
        ErrorKind::CannotTagContainingClass,
        format!("invalid tag on member `{}`: tagged members cannot contain classes", identifier),
        identifier
    ),
    (
        "E023",
        ErrorKind::TypeMismatch,
        format!("type mismatch: expected a `{expected}` but found a {found} (which doesn't implement `{expected}`)"),
        expected,
        found
    ),
    (
        "E024",
        ErrorKind::ConcreteTypeMismatch,
        format!("type mismatch: expected `{expected}` but found a `{found}`"),
        expected,
        found
    ),
    (
        "E025",
        ErrorKind::CompactStructCannotBeEmpty,
        "compact structs must be non-empty"
    ),
    (
        "E026",
        ErrorKind::SelfReferentialTypeAliasNeedsConcreteType,
        format!("self-referential type alias '{}' has no concrete type", identifier),
        identifier
    ),
    (
        "E027",
        ErrorKind::EnumeratorValueOutOfBounds,
        format!(
            "invalid enumerator `{identifier}`: enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive",
        ),
        identifier, value, min, max
    ),
    (
        "E028",
        ErrorKind::TagValueOutOfBounds,
        "tag values must be within the range 0 <= value <= 2147483647"
    ),
    (
        "E029",
        ErrorKind::DuplicateEnumeratorValue,
        format!("enumerator values must be unique; the value `{}` is already in use", value),
        value
    ),
    (
        "E030",
        ErrorKind::NotSupportedWithEncoding,
        format!("{kind} `{identifier}` is not supported by the {encoding} encoding"),
        kind, identifier, encoding
    ),
    (
        "E031",
        ErrorKind::UnsupportedType,
        format!("the type `{type_string}` is not supported by the {encoding} encoding"),
        type_string,
        encoding
    ),
    (
        "E032",
        ErrorKind::ExceptionNotSupported,
        format!("exceptions cannot be used as a data type with the {encoding} encoding"),
        encoding
    ),
    (
        "E033",
        ErrorKind::OptionalsNotSupported,
        format!("optional types are not supported by the {encoding} encoding (except for classes, proxies, and with tags)"),
        encoding
    ),
    (
        "E034",
        ErrorKind::StreamedParametersNotSupported,
        format!("streamed parameters are not supported by the {encoding} encoding"),
        encoding
    ),
    (
        "E035",
        ErrorKind::UnexpectedAttribute,
        format!("unexpected attribute `{attribute}`"),
        attribute
    ),
    (
        "E036",
        ErrorKind::MissingRequiredArgument,
        format!("missing required argument `{argument}`"),
        argument
    ),
    (
        "E037",
        ErrorKind::TooManyArguments,
        format!("too many arguments, expected `{expected}`"),
        expected
    ),
    (
        "E038",
        ErrorKind::MissingRequiredAttribute,
        format!("missing required attribute `{attribute}`"),
        attribute
    ),
    (
        "E039",
        ErrorKind::MultipleStreamedMembers,
        "cannot have multiple streamed members"
    ),
    (
        "E040",
        ErrorKind::InvalidAttribute,
        format!("attribute `{attribute}` cannot be used on `{kind}`"),
        attribute,
        kind
    ),
    (
        "E041",
        ErrorKind::CompactIdOutOfBounds,
        "compact IDs must be within the range 0 <= ID <= 2147483647"
    ),
    (
        "E042",
        ErrorKind::IntegerLiteralOverflows,
        "integer literal is outside the parsable range of -2^127 <= i <= 2^127 - 1"
    ),
    (
        "E043",
        ErrorKind::InvalidIntegerLiteral,
        format!("integer literal contains illegal characters for base-{base}"),
        base
    ),
    (
        "E044",
        ErrorKind::InvalidEncodingVersion,
        format!("'{version}' is not a valid Slice encoding version"),
        version
    ),
    (
        "E046",
        ErrorKind::MultipleEncodingVersions,
        "only a single encoding can be specified per file".to_owned()
    ),
    (
        "E047",
        ErrorKind::FileScopedModuleCannotContainSubModules,
        format!("file scoped module `{identifier}` cannot contain sub modules"),
        identifier
    ),
    (
        "E048",
        ErrorKind::AnyExceptionNotSupported,
        format!("operations that throw AnyException are only supported by the Slice1 encoding")

    ),
    (
        "E049",
        ErrorKind::InvalidWarningCode,
        format!("the warning code `{code}` is not valid"),
        code
    )
);
