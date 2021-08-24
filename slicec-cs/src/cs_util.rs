// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::ref_from_node;
use slice::util::{fix_case, CaseStyle, TypeContext};
use slice::writer::Writer;

// TODOAUSTIN move this function beneath the other functions.
pub fn return_type_to_string(
    return_type: &ReturnType,
    scope: &str,
    ast: &Ast,
    context: TypeContext,
) -> String {
    let mut type_string = "global::System.Threading.Tasks.ValueTask".to_owned();
    match return_type {
        ReturnType::Void(_) => {}
        ReturnType::Single(data_type, _) => {
            let type_node = data_type.definition(ast);
            type_string += "<";
            type_string += &type_to_string(type_node, scope, ast, context);
            type_string += ">";
        }
        ReturnType::Tuple(tuple, _) => {
            type_string += "<(";
            for id in tuple.iter() {
                let return_element = ref_from_node!(Node::Member, ast, *id);
                let data_type = return_element.data_type.definition(ast);
                type_string += format!(
                    "{} {}, ",
                    type_to_string(data_type, scope, ast, context),
                    return_element.identifier(),
                ).as_str();
            }
            // Remove the trailing comma and space.
            type_string.truncate(type_string.len() - 2);
            type_string += ")>";
        }
    };
    type_string
}

pub fn type_to_string(node: &Node, scope: &str, ast: &Ast, context: TypeContext) -> String {
    match node {
        Node::Struct(_, struct_def) => {
            let identifier = escape_scoped_identifier(struct_def, CaseStyle::Pascal);
            fix_scope(&identifier, scope)
        }
        Node::Interface(_, interface_def) => {
            let identifier = escape_scoped_identifier(interface_def, CaseStyle::Pascal) + "Prx";
            fix_scope(&identifier, scope)
        }
        Node::Enum(_, enum_def) => {
            let identifier = escape_scoped_identifier(enum_def, CaseStyle::Pascal);
            fix_scope(&identifier, scope)
        }
        Node::Sequence(_, sequence) => sequence_type_to_string(sequence, scope, ast, context),
        Node::Dictionary(_, dictionary) => {
            dictionary_type_to_string(dictionary, scope, ast, context)
        }
        Node::Primitive(_, primitive) => match primitive {
            Primitive::Bool     => "bool",
            Primitive::Byte     => "byte",
            Primitive::Short    => "short",
            Primitive::UShort   => "ushort",
            Primitive::Int      => "int",
            Primitive::UInt     => "uint",
            Primitive::VarInt   => "int",
            Primitive::VarUInt  => "uint",
            Primitive::Long     => "long",
            Primitive::ULong    => "ulong",
            Primitive::VarLong  => "long",
            Primitive::VarULong => "ulong",
            Primitive::Float    => "float",
            Primitive::Double   => "double",
            Primitive::String   => "string",
        }.to_owned(),
        _ => {
            panic!("Node does not represent a type: '{:?}'!", node);
        }
    }
}

fn sequence_type_to_string(
    sequence: &Sequence,
    scope: &str,
    ast: &Ast,
    context: TypeContext,
) -> String {
    let element_type = sequence.element_type.definition(ast);
    let element_type_string = type_to_string(element_type, scope, ast, TypeContext::Nested);

    match context {
        TypeContext::DataMember | TypeContext::Nested => {
            format!("global::System.Collections.Generic.IList<{}>", element_type_string)
        }
        TypeContext::Incoming => {
            format!("{}[]", element_type_string)
        }
        TypeContext::Outgoing => {
            // If the underlying type is of fixed size, we map to `ReadOnlyMemory` instead.
            if element_type.as_type().unwrap().is_fixed_size(ast) {
                format!("global::System.Collections.Generic.IEnumerable<{}>", element_type_string)
            } else {
                format!("global::System.ReadOnlyMemory<{}>", element_type_string)
            }
        }
    }
}

fn dictionary_type_to_string(
    dictionary: &Dictionary,
    scope: &str,
    ast: &Ast,
    context: TypeContext,
) -> String {
    let key_type = dictionary.key_type.definition(ast);
    let value_type = dictionary.value_type.definition(ast);
    let key_type_string = type_to_string(key_type, scope, ast, TypeContext::Nested);
    let value_type_string = type_to_string(value_type, scope, ast, TypeContext::Nested);

    match context {
        TypeContext::DataMember | TypeContext::Nested => {
            format!(
                "global::System.Collections.Generic.IDictionary<{}, {}>",
                key_type_string, value_type_string,
            )
        }
        TypeContext::Incoming => {
            format!(
                "global::System.Collections.Generic.Dictionary<{}, {}>",
                key_type_string, value_type_string,
            )
        }
        TypeContext::Outgoing => {
            format!(
                "global::System.Collections.Generic.IEnumerable<global::System.Collections.Generic.KeyValuePair<{}, {}>>",
                key_type_string, value_type_string,
            )
        }
    }
}

/// Escapes and returns the definition's identifier, without any scoping.
/// If the identifier is a C# keyword, a '@' prefix is appended to it.
pub fn escape_identifier(definition: &dyn NamedSymbol, case: CaseStyle) -> String {
    escape_keyword(&fix_case(definition.identifier(), case))
}

/// Escapes and returns the definition's identifier, fully scoped.
/// If the identifier or any of the scopes are C# keywords, a '@' prefix is appended to them.
/// Note: The case style is applied to all scope segments, not just the last one.
pub fn escape_scoped_identifier(definition: &dyn NamedSymbol, case: CaseStyle) -> String {
    let mut scoped_identifier = String::new();

    // Escape any keywords in the scope identifiers.
    // We skip the first scope segment, since it is always an empty string because all scopes start
    // with '::' (to represent global scope).
    for segment in definition.scope().split("::").skip(1) {
        scoped_identifier += &(escape_keyword(&fix_case(segment, case)) + ".");
    }
    scoped_identifier += &escape_identifier(definition, case);
    scoped_identifier
}

/// Checks if the provided string is a C# keyword, and escapes it if necessary (by appending a '@').
pub fn escape_keyword(identifier: &str) -> String {
    const CS_KEYWORDS: [&str; 79] = [
        "abstract", "as", "async", "await", "base", "bool", "break", "byte", "case", "catch",
        "char", "checked", "class", "const", "continue", "decimal", "default", "delegate", "do",
        "double", "else", "enum", "event", "explicit", "extern", "false", "finally", "fixed",
        "float", "for", "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal",
        "is", "lock", "long", "namespace", "new", "null", "object", "operator", "out", "override",
        "params", "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed",
        "short", "sizeof", "stackalloc", "static", "string", "struct", "switch", "this", "throw",
        "true", "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using",
        "virtual", "void", "volatile", "while",
    ];

    // Add a '@' prefix if the identifier matched a C# keyword.
    (if CS_KEYWORDS.contains(&identifier) { "@" } else { "" }.to_owned()) + identifier
}

// TODOAUSTIN WE NEED TO HANDLE NAME MANGLING FOR CLASSES AND EXCEPTIONS!
/// Checks if the provided identifier would shadow a base method in an object or exception, and
/// escapes it if necessary by appending an 'Ice' prefix to the identifier.
///
/// `kind` is the stringified slice type. Escaping is only performed on `class`es and `exception`s.
/// TODOAUSTIN write a better comment
pub fn mangle_name(identifier: &str, kind: &str) -> String {
    // The names of all the methods defined on the Object base class.
    const OBJECT_BASE_NAMES: [&str; 7] = [
        "Equals", "Finalize", "GetHashCode", "GetType", "MemberwiseClone", "ReferenceEquals",
        "ToString",
    ];
    // The names of all the methods and properties defined on the Exception base class.
    const EXCEPTION_BASE_NAMES: [&str; 10] = [
        "Data", "GetBaseException", "GetObjectData", "HelpLink", "HResult", "InnerException",
        "Message", "Source", "StackTrace", "TargetSite",
    ];

    let needs_mangling = match kind {
        "exception" => {
            OBJECT_BASE_NAMES.contains(&identifier) | EXCEPTION_BASE_NAMES.contains(&identifier)
        }
        "class" => OBJECT_BASE_NAMES.contains(&identifier),
        _ => false,
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
        if !unscoped_identifier.contains(".") {
            return unscoped_identifier.to_owned();
        }
    }

    if scoped_identifier.starts_with("IceRpc") {
        scoped_identifier.to_owned()
    } else {
        "global::".to_owned() + scoped_identifier
    }
}

pub fn write_equality_operators(writer: &mut Writer, name: &str) {
    writer.write_line_separator();
    let content = format!(
        r#"
/// <summary>The equality operator == returns true if its operands are equal, false otherwise.</summary>
/// <param name="lhs">The left hand side operand.</param>
/// <param name="rhs">The right hand side operand.</param>
/// <returns><c>true</c> if the operands are equal, otherwise <c>false</c>.</returns>
public static bool operator ==({name} lhs, {name} rhs) => lhs.Equals(rhs);

/// <summary>The inequality operator != returns true if its operands are not equal, false otherwise.</summary>"
/// <param name="lhs">The left hand side operand.</param>
/// <param name="rhs">The right hand side operand.</param>
/// <returns><c>true</c> if the operands are not equal, otherwise <c>false</c>.</returns>
public static bool operator !=({name} lhs, {name} rhs) => !lhs.Equals(rhs);"#,
        name = name
    );
    writer.write(&content);
}

pub fn builtin_suffix(node: &Node) -> String {
    match node {
        Node::Primitive(_, primitive) => primitive_type_suffix(primitive),
        // Node::Proxy(_, proxy) => "Proxy".to_owned(), //TODO: proxies
        // Node::Class(class: &Class) => "Class" //TODO: classes
        _ => panic!("unexpected builtin type: {}", node.as_element().kind()),
    }
}

pub fn primitive_type_suffix(primitive: &Primitive) -> String {
    // TODO: can we just stringify the primitive?
    match primitive {
        Primitive::Bool => "Bool",
        Primitive::Byte => "Byte",
        Primitive::Short => "Short",
        Primitive::UShort => "UShort",
        Primitive::Int => "Int",
        Primitive::UInt => "UInt",
        Primitive::VarInt => "VarInt",
        Primitive::VarUInt => "VarUInt",
        Primitive::Long => "Long",
        Primitive::ULong => "",
        Primitive::VarLong => "VarLong",
        Primitive::VarULong => "VarULong",
        Primitive::Float => "Float",
        Primitive::Double => "Double",
        Primitive::String => "String",
    }
    .to_owned()
}

pub fn get_scoped_unqualified(_: &Node, _: &str, _: &Ast) -> String {
    "".to_owned()
}

pub fn get_unqualified(_: &Node, _: &str, _: &str, _: &str, _: &Ast) -> String {
    "".to_owned()
}

pub fn helper_name(type_ref: &TypeRef, scope: &str, ast: &Ast) -> String {
    get_unqualified(type_ref.definition(ast), scope, "", "Helper", ast)
}

pub fn field_name(member: &Member) -> String {
    let identifier = member.identifier();
    // TODO: port this C++ code
    // string name = member->name();
    // return normalizeCase(member) ? fixId(pascalCase(name)) : fixId(name);

    identifier.to_owned()
}

pub fn is_value_type(type_ref: &TypeRef, ast: &Ast) -> bool {
    match type_ref.definition(ast) {
        Node::Primitive(_, primitive) => !matches!(primitive, Primitive::String),
        Node::Enum(_, _) | Node::Struct(_, _) | Node::Interface(_, _) => true,
        _ => false,
    }
}

pub fn is_reference_type(type_ref: &TypeRef, ast: &Ast) -> bool {
    !is_value_type(type_ref, ast)
}
