// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::ref_from_node;
use slice::util::TypeContext;
use slice::writer::Writer;

// TODO move this function beneath the other functions.
pub fn return_type_to_string(return_type: &ReturnType, ast: &Ast, context: TypeContext) -> String {
    let mut type_string = "global::System.Threading.Tasks.ValueTask".to_owned();
    match return_type {
        ReturnType::Void(_) => {}
        ReturnType::Single(data_type, _) => {
            let node = ast.resolve_index(data_type.definition.unwrap());
            type_string += "<";
            type_string += &type_to_string(node, ast, context);
            type_string += ">";
        }
        ReturnType::Tuple(tuple, _) => {
            type_string += "<(";
            for id in tuple.iter() {
                let return_element = ref_from_node!(Node::Member, ast, *id);
                let data_type = ast.resolve_index(return_element.data_type.definition.unwrap());
                type_string += format!(
                    "{} {}, ",
                    type_to_string(data_type, ast, context),
                    return_element.identifier(),
                )
                .as_str();
            }
            // Remove the trailing comma and space.
            type_string.truncate(type_string.len() - 2);
            type_string += ")>";
        }
    };
    type_string
}

pub fn type_to_string(node: &Node, ast: &Ast, context: TypeContext) -> String {
    match node {
        Node::Struct(_, struct_def) => {
            let mut identifier = struct_def.scope.clone().unwrap() + "::" + struct_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        }
        Node::Interface(_, interface_def) => {
            let mut identifier =
                interface_def.scope.clone().unwrap() + "::" + interface_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".") + "Prx"
        }
        Node::Enum(_, enum_def) => {
            let mut identifier = enum_def.scope.clone().unwrap() + "::" + enum_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        }
        Node::Sequence(_, sequence) => sequence_type_to_string(sequence, ast, context),
        Node::Dictionary(_, dictionary) => dictionary_type_to_string(dictionary, ast, context),
        Node::Primitive(_, primitive) => match primitive {
            Primitive::Bool => "bool",
            Primitive::Byte => "byte",
            Primitive::Short => "short",
            Primitive::UShort => "ushort",
            Primitive::Int => "int",
            Primitive::UInt => "uint",
            Primitive::VarInt => "int",
            Primitive::VarUInt => "uint",
            Primitive::Long => "long",
            Primitive::ULong => "ulong",
            Primitive::VarLong => "long",
            Primitive::VarULong => "ulong",
            Primitive::Float => "float",
            Primitive::Double => "double",
            Primitive::String => "string",
        }
        .to_owned(),
        _ => {
            panic!("Node does not represent a type: '{:?}'!", node);
        }
    }
}

fn sequence_type_to_string(sequence: &Sequence, ast: &Ast, context: TypeContext) -> String {
    let element_type = ast.resolve_index(sequence.element_type.definition.unwrap());
    let element_type_string = type_to_string(element_type, ast, TypeContext::Nested);

    match context {
        TypeContext::DataMember | TypeContext::Nested => {
            format!(
                "global::System.Collections.Generic.IList<{}>",
                element_type_string,
            )
        }
        TypeContext::Incoming => {
            format!("{}[]", element_type_string,)
        }
        TypeContext::Outgoing => {
            let mut container_type = "global::System.Collections.Generic.IEnumerable";
            // If the underlying type is of fixed size, we map to `ReadOnlyMemory` instead.
            if element_type.as_type().unwrap().is_fixed_size(ast) {
                container_type = "global::System.ReadOnlyMemory";
            }
            format!("{}<{}>", container_type, element_type_string,)
        }
    }
}

fn dictionary_type_to_string(dictionary: &Dictionary, ast: &Ast, context: TypeContext) -> String {
    let key_type = ast.resolve_index(dictionary.key_type.definition.unwrap());
    let value_type = ast.resolve_index(dictionary.value_type.definition.unwrap());
    let key_type_string = type_to_string(key_type, ast, TypeContext::Nested);
    let value_type_string = type_to_string(value_type, ast, TypeContext::Nested);

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
                key_type_string,
                value_type_string,
            )
        }
    }
}

// TODO write a doc comment here.
pub fn escape_identifier(definition: &dyn NamedSymbol) -> String {
    // Only NamedSymbols have identifiers, so this unwrap will always succeed.
    let identifier = definition.identifier();

    const cs_keywords: [&'static str; 79] = [
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

    // Check if the identifier is a C# keyword.
    if cs_keywords.iter().find(|&&keyword| identifier == keyword) {
        "@".to_owned() + identifier
    } else {
        // Otherwise check if it would shadow a base method name.
        mangle_name(definition, identifier)
    }
}

// TODO write a doc comment here.
fn mangle_name(definition: &dyn NamedSymbol, identifier: &str) -> String {
    // The names of all the methods defined on the Object base class.
    const object_base_names: [&'static str; 7] = [
        "Equals", "Finalize", "GetHashCode", "GetType", "MemberwiseClone", "ReferenceEquals",
        "ToString",
    ];
    // The names of all the methods and properties defined on the Exception base class.
    const exception_base_names: [&'static str; 10] = [
        "Data", "GetBaseException", "GetObjectData", "HelpLink", "HResult", "InnerException",
        "Message", "Source", "StackTrace", "TargetSite",
    ];

    let mut needs_mangling = false;
    match definition.kind() {
        // TODO add checks for classes and exceptions once we've added them.
        //"class" => {
        //    needs_mangling =
        //        object_base_names.iter().find(|&&name| identifier == name).is_some();
        //}
        //"exception" => {
        //    needs_mangling =
        //        object_base_names.iter().find(|&&name| identifier == name).is_some() |
        //        exception_base_names.iter().find(|&&name| identifier == name).is_some();
        //}
    }

    // If the name conflicts with a base method, add an "Ice" prefix to it.
    (if needs_mangling { "Ice" } else { "" }).to_owned() + identifier;
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

pub fn decode_data_members(struct_def: &Struct, ast: &Ast) -> String {
    let mut content = String::new();
    for id in &struct_def.contents {
        let member = ref_from_node!(Node::Member, ast, *id);
        let identifier = member.identifier();
        // let type_node = ast.resolve_index(member.data_type.definition.unwrap());
        // let type_string = type_to_string(type_node, ast, TypeContext::DataMember);

        content += &format!(
            "{}this.{identifier} = decoder.Decode",
            if content.len() > 0 { "\n" } else { "" },
            identifier = identifier
        );
    }

    content
}

pub fn builtin_suffix(node: &Node) -> String {
    match node {
        Node::Primitive(_, primitive) => primitive_type_suffix(&primitive),
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
