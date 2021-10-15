// Copyright (c) ZeroC, Inc. All rights reserved.

/// Checks if the provided string is a C# keyword, and escapes it if necessary (by appending a '@').
pub fn escape_keyword(identifier: &str) -> String {
    const CS_KEYWORDS: [&str; 79] = [
        "abstract",
        "as",
        "async",
        "await",
        "base",
        "bool",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "checked",
        "class",
        "const",
        "continue",
        "decimal",
        "default",
        "delegate",
        "do",
        "double",
        "else",
        "enum",
        "event",
        "explicit",
        "extern",
        "false",
        "finally",
        "fixed",
        "float",
        "for",
        "foreach",
        "goto",
        "if",
        "implicit",
        "in",
        "int",
        "interface",
        "internal",
        "is",
        "lock",
        "long",
        "namespace",
        "new",
        "null",
        "object",
        "operator",
        "out",
        "override",
        "params",
        "private",
        "protected",
        "public",
        "readonly",
        "ref",
        "return",
        "sbyte",
        "sealed",
        "short",
        "sizeof",
        "stackalloc",
        "static",
        "string",
        "struct",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "uint",
        "ulong",
        "unchecked",
        "unsafe",
        "ushort",
        "using",
        "virtual",
        "void",
        "volatile",
        "while",
    ];

    // Add a '@' prefix if the identifier matched a C# keyword.
    (if CS_KEYWORDS.contains(&identifier) { "@" } else { "" }.to_owned()) + identifier
}

// TODOAUSTIN comment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    NonMangled,
    Class,
    Exception,
}

// TODOAUSTIN WE NEED TO HANDLE NAME MANGLING FOR CLASSES AND EXCEPTIONS!
/// Checks if the provided identifier would shadow a base method in an object or exception, and
/// escapes it if necessary by appending an 'Ice' prefix to the identifier.
///
/// `kind` is the stringified C# type. Escaping is only performed on `object`es and `exception`s.
/// TODOAUSTIN write a better comment
pub fn mangle_name(identifier: &str, field_type: FieldType) -> String {
    // The names of all the methods defined on the Object base class.
    const OBJECT_BASE_NAMES: [&str; 7] = [
        "Equals",
        "Finalize",
        "GetHashCode",
        "GetType",
        "MemberwiseClone",
        "ReferenceEquals",
        "ToString",
    ];
    // The names of all the methods and properties defined on the Exception base class.
    const EXCEPTION_BASE_NAMES: [&str; 10] = [
        "Data",
        "GetBaseException",
        "GetObjectData",
        "HelpLink",
        "HResult",
        "InnerException",
        "Message",
        "Source",
        "StackTrace",
        "TargetSite",
    ];

    let needs_mangling = match field_type {
        FieldType::Exception => {
            OBJECT_BASE_NAMES.contains(&identifier) | EXCEPTION_BASE_NAMES.contains(&identifier)
        }
        FieldType::Class => OBJECT_BASE_NAMES.contains(&identifier),
        FieldType::NonMangled => false,
    };

    // If the name conflicts with a base method, add an "Ice" prefix to it.
    (if needs_mangling { "Ice" } else { "" }).to_owned() + identifier
}

/// TODO write a comment here!
/// THIS IS ONLY FOR NON_BUILTIN TYPES! NO PRIMITIVES, NO SEQUENCES, and NO DICTIONARIES!
pub fn fix_scope(scoped_identifier: &str, current_scope: &str) -> String {
    let scope_prefix = current_scope.to_owned() + ".";
    // Check if `scoped_identifier` starts with `current_scope`, and strip it off.
    if let Some(unscoped_identifier) = scoped_identifier.strip_prefix(&scope_prefix) {
        // If the identifier is now fully unscoped, return the fully unscoped identifier.
        if !unscoped_identifier.contains('.') {
            return unscoped_identifier.to_owned();
        }
    }

    if scoped_identifier.starts_with("IceRpc") {
        scoped_identifier.to_owned()
    } else {
        "global::".to_owned() + scoped_identifier
    }
}
