// Copyright (c) ZeroC, Inc.

use super::{DiagnosticReporter, Note};
use crate::grammar::Encoding;
use crate::implement_diagnostic_functions;
use crate::slice_file::Span;
use in_definite;

#[derive(Debug)]
pub struct Error {
    pub(super) kind: ErrorKind,
    pub(super) span: Option<Span>,
    pub(super) notes: Vec<Note>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            span: None,
            notes: Vec::new(),
        }
    }

    // TODO only used by consumers of `find_element` and `find_node`,
    // we should add a more specific error type for those functions to use.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn set_span(mut self, span: &Span) -> Self {
        self.span = Some(span.to_owned());
        self
    }

    pub fn add_note(mut self, message: impl Into<String>, span: Option<&Span>) -> Self {
        self.notes.push(Note {
            message: message.into(),
            span: span.cloned(),
        });
        self
    }

    pub fn add_notes(mut self, notes: Vec<Note>) -> Self {
        self.notes.extend(notes);
        self
    }

    pub fn report(self, diagnostic_reporter: &mut DiagnosticReporter) {
        diagnostic_reporter.report(self);
    }

    pub fn error_code(&self) -> Option<&str> {
        self.kind.error_code()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind.message())
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    // ----------------  Generic Errors ---------------- //
    IO {
        action: &'static str,
        path: String,
        error: std::io::Error,
    },

    Syntax {
        message: String,
    },

    // ----------------  Attribute Errors ---------------- //
    /// Used to indicate when the compress attribute cannot be applied.
    CompressAttributeCannotBeApplied,

    /// Used to indicate when the deprecated attribute cannot be applied.
    DeprecatedAttributeCannotBeApplied {
        /// The kind which the deprecated attribute was applied to.
        kind: String,
    },

    // ----------------  Argument Errors ---------------- //
    /// The provided argument is not supported for the given method.
    ArgumentNotSupported {
        /// The name of the argument.
        argument_name: String,
        /// The name of the method.
        method_name: String,
    },

    // ---------------- Dictionary Errors ---------------- //
    /// Dictionaries cannot use optional types as keys.
    KeyMustBeNonOptional,

    /// An unsupported type was used as a dictionary key type.
    KeyTypeNotSupported {
        /// The identifier of the type that was used as a dictionary key type.
        identifier: String,
    },

    /// Struct contains a member that cannot be used as a dictionary key type.
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

    /// The required parameters of an operation did not precede the optional parameters.
    RequiredMustPrecedeOptional {
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

    /// Compact structs cannot contain tagged data members.
    CompactStructCannotContainTaggedMembers,

    // ----------------  Tag Errors ---------------- //
    /// A duplicate tag value was found.
    CannotHaveDuplicateTag {
        /// The identifier of the tagged member.
        member_identifier: String,
    },

    /// Cannot tag a class.
    CannotTagClass {
        /// The identifier of the tagged member.
        member_identifier: String,
    },

    /// Cannot tag a member that contains a class.
    CannotTagContainingClass {
        /// The identifier of the tagged member.
        member_identifier: String,
    },

    /// A tag value was not in the expected range, 0 .. i32::MAX.
    TagValueOutOfBounds,

    /// A tagged data member was not set to optional.
    TaggedMemberMustBeOptional {
        /// The identifier of the tagged member.
        member_identifier: String,
    },

    // ----------------  General Errors ---------------- //
    /// A compact ID was not in the expected range, 0 .. i32::MAX.
    CompactIdOutOfBounds,

    /// Used to indicate when a method must contain arguments.
    CannotBeEmpty {
        /// The name of the method.
        member_identifier: String,
    },

    /// Used to indicate when two concrete types should match, but do not.
    ConcreteTypeMismatch {
        /// The name of the expected kind.
        expected: String,
        /// The name of the found kind.
        kind: String,
    },

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
        encoding: i128,
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

    /// Failed to resolve a type due to a cycle in its definition.
    CannotResolveDueToCycles,

    /// No element with the specified identifier was found.
    DoesNotExist {
        /// The identifier that was not found.
        identifier: String,
    },

    // ----------------  Attribute Errors ---------------- //
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
}

implement_diagnostic_functions!(
    ErrorKind,
    (
        ErrorKind::IO,
        format!("failed to {action} '{path}': {error}"),
        action,
        path,
        error
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
        format!("{member_identifier} arguments cannot be empty"),
        member_identifier
    ),
    (
        "E004",
        ErrorKind::ArgumentNotSupported,
        format!("argument '{argument_name}' is not supported for '{method_name}'"),
        argument_name,
        method_name
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
        format!("struct '{struct_identifier}' contains members that cannot be used as a dictionary key type"),
        struct_identifier
    ),
    (
        "E009",
        ErrorKind::CannotUseOptionalUnderlyingType,
        format!("invalid enum '{enum_identifier}': enums cannot have optional underlying types"),
        enum_identifier
    ),
    (
        "E010",
        ErrorKind::MustContainEnumerators,
        format!("invalid enum '{enum_identifier}': enums must contain at least one enumerator"),
        enum_identifier
    ),
    (
        "E011",
        ErrorKind::UnderlyingTypeMustBeIntegral,
        format!("invalid enum '{enum_identifier}': underlying type '{kind}' is not supported for enums"),
        enum_identifier,
        kind
    ),
    (
        "E012",
        ErrorKind::Redefinition,
        format!("redefinition of '{identifier}'"),
        identifier
    ),
    (
        "E013",
        ErrorKind::Shadows,
        format!("'{identifier}' shadows another symbol"),
        identifier
    ),
    (
        "E014",
        ErrorKind::CannotHaveDuplicateTag,
        format!("invalid tag on member '{member_identifier}': tags must be unique"),
        member_identifier
    ),
    (
        "E015",
        ErrorKind::RequiredMustPrecedeOptional,
        format!("invalid parameter '{parameter_identifier}': required parameters must precede tagged parameters"),
        parameter_identifier
    ),
    (
        "E016",
        ErrorKind::StreamedMembersMustBeLast,
        format!("invalid parameter '{parameter_identifier}': only the last parameter in an operation can use the stream modifier"),
        parameter_identifier
    ),
    (
        "E017",
        ErrorKind::ReturnTuplesMustContainAtLeastTwoElements,
        "return tuples must have at least 2 elements"
    ),
    (
        "E018",
        ErrorKind::CompactStructCannotContainTaggedMembers,
        "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
    ),
    (
        "E019",
        ErrorKind::TaggedMemberMustBeOptional,
        format!("invalid tag on member '{member_identifier}': tagged members must be optional"),
        member_identifier
    ),
    (
        "E020",
        ErrorKind::CannotTagClass,
        format!("invalid tag on member '{member_identifier}': tagged members cannot be classes"),
        member_identifier
    ),
    (
        "E021",
        ErrorKind::CannotTagContainingClass,
        format!("invalid tag on member '{member_identifier}': tagged members cannot contain classes"),
        member_identifier
    ),
    (
        "E022",
        ErrorKind::TypeMismatch,
        format!(
            "type mismatch: expected {} '{expected}' but found a {actual} (which doesn't implement '{expected}')",
            in_definite::get_a_or_an(expected)
        ),
        expected,
        actual
    ),
    (
        "E023",
        ErrorKind::ConcreteTypeMismatch,
        format!(
            "type mismatch: expected {} '{expected}' but found {} '{kind}'",
            in_definite::get_a_or_an(expected),
            in_definite::get_a_or_an(kind)
        ),
        expected,
        kind
    ),
    (
        "E024",
        ErrorKind::CompactStructCannotBeEmpty,
        "compact structs must be non-empty"
    ),
    (
        "E025",
        ErrorKind::SelfReferentialTypeAliasNeedsConcreteType,
        format!("self-referential type alias '{identifier}' has no concrete type"),
        identifier
    ),
    (
        "E026",
        ErrorKind::EnumeratorValueOutOfBounds,
        format!(
            "invalid enumerator '{enumerator_identifier}': enumerator value '{value}' is out of bounds. The value must be between '{min}..{max}', inclusive",
        ),
        enumerator_identifier, value, min, max
    ),
    (
        "E027",
        ErrorKind::TagValueOutOfBounds,
        "tag values must be within the range 0 <= value <= 2147483647"
    ),
    (
        "E028",
        ErrorKind::DuplicateEnumeratorValue,
        format!("enumerator values must be unique; the value '{enumerator_value}' is already in use"),
        enumerator_value
    ),
    (
        "E029",
        ErrorKind::NotSupportedWithEncoding,
        format!("{kind} '{identifier}' is not supported by the {encoding} encoding"),
        kind, identifier, encoding
    ),
    (
        "E030",
        ErrorKind::UnsupportedType,
        format!("the type '{kind}' is not supported by the {encoding} encoding"),
        kind,
        encoding
    ),
    (
        "E031",
        ErrorKind::ExceptionNotSupported,
        format!("exceptions cannot be used as a data type with the {encoding} encoding"),
        encoding
    ),
    (
        "E032",
        ErrorKind::OptionalsNotSupported,
        format!("optional types are not supported by the {encoding} encoding (except for classes, proxies, and with tags)"),
        encoding
    ),
    (
        "E033",
        ErrorKind::StreamedParametersNotSupported,
        format!("streamed parameters are not supported by the {encoding} encoding"),
        encoding
    ),
    (
        "E034",
        ErrorKind::UnexpectedAttribute,
        format!("unexpected attribute '{attribute}'"),
        attribute
    ),
    (
        "E035",
        ErrorKind::MissingRequiredArgument,
        format!("missing required argument '{argument}'"),
        argument
    ),
    (
        "E036",
        ErrorKind::TooManyArguments,
        format!("too many arguments, expected '{expected}'"),
        expected
    ),
    (
        "E037",
        ErrorKind::MissingRequiredAttribute,
        format!("missing required attribute '{attribute}'"),
        attribute
    ),
    (
        "E038",
        ErrorKind::MultipleStreamedMembers,
        "cannot have multiple streamed members"
    ),
    (
        "E039",
        ErrorKind::CompactIdOutOfBounds,
        "compact IDs must be within the range 0 <= ID <= 2147483647"
    ),
    (
        "E040",
        ErrorKind::IntegerLiteralOverflows,
        "integer literal is outside the parsable range of -2^127 <= i <= 2^127 - 1"
    ),
    (
        "E041",
        ErrorKind::InvalidIntegerLiteral,
        format!("integer literal contains illegal characters for base-{base}"),
        base
    ),
    (
        "E042",
        ErrorKind::InvalidEncodingVersion,
        format!("'{encoding}' is not a valid Slice encoding version"),
        encoding
    ),
    (
        "E043",
        ErrorKind::MultipleEncodingVersions,
        "only a single encoding can be specified per file".to_owned()
    ),
    (
        "E044",
        ErrorKind::FileScopedModuleCannotContainSubModules,
        format!("file scoped module '{identifier}' cannot contain sub modules"),
        identifier
    ),
    (
        "E045",
        ErrorKind::AnyExceptionNotSupported,
        format!("operations that throw AnyException are only supported by the Slice1 encoding")

    ),
    (
        "E046",
        ErrorKind::InvalidWarningCode,
        format!("the warning code '{code}' is not valid"),
        code
    ),
    (
        "E047",
        ErrorKind::InfiniteSizeCycle,
        format!("self-referential type {type_id} has infinite size.\n{cycle}"),
        type_id, cycle
    ),
    (
        "E048",
        ErrorKind::CannotResolveDueToCycles,
        "failed to resolve type due to a cycle in its definition".to_owned()
    ),
    (
        "E049",
        ErrorKind::DoesNotExist,
        format!("no element with identifier '{identifier}' exists"),
        identifier
    ),
    (
        "E050",
        ErrorKind::AttributeIsNotRepeatable,
        format!("duplicate attribute '{attribute}'"),
        attribute
    )
);
