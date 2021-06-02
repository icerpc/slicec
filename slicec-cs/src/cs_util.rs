// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ref_from_node;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::TypeContext;

pub fn return_type_to_string(return_type: &ReturnType, ast: &Ast) -> String {
    let mut type_string = "global::System.Threading.Tasks.ValueTask".to_owned();
    match return_type {
        ReturnType::Void(_) => {}
        ReturnType::Single(data_type, _) => {
            let node = ast.resolve_index(data_type.definition.unwrap());
            type_string += "<";
            type_string += &type_to_string(node, ast, TypeContext::ReturnParameter);
            type_string += ">";
        }
        ReturnType::Tuple(tuple, _) => {
            type_string += "<(";
            for id in tuple.iter() {
                let parameter = ref_from_node!(Node::Parameter, ast, *id);
                let data_type = ast.resolve_index(parameter.data_type.definition.unwrap());
                type_string += format!(
                    "{} {}, ",
                    type_to_string(data_type, ast, TypeContext::ReturnParameter),
                    parameter.identifier(),
                ).as_str();
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
            let mut identifier = interface_def.scope.clone().unwrap() + "::" + interface_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".") + "Prx"
        }
        Node::Enum(_, enum_def) => {
            let mut identifier = enum_def.scope.clone().unwrap() + "::" + enum_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        }
        Node::Sequence(_, sequence) => {
            sequence_type_to_string(sequence, ast, context)
        }
        Node::Dictionary(_, dictionary) => {
            dictionary_type_to_string(dictionary, ast, context)
        }
        Node::Primitive(_, primitive) => {
            match primitive {
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
            }.to_owned()
        }
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
        TypeContext::InParameter => {
            format!(
                "{}[]",
                element_type_string,
            )
        }
        TypeContext::ReturnParameter => {
            let mut container_type = "global::System.Collections.Generic.IEnumerable";
            // If the underlying type is a fixed size primitive, we map to `ReadOnlyMemory` instead.
            if let Node::Primitive(_, primitive) = element_type {
                if *primitive != Primitive::String {
                    container_type = "global::System.ReadOnlyMemory";
                }
            }
            format!(
                "{}<{}>",
                container_type,
                element_type_string,
            )
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
                key_type_string,
                value_type_string,
            )
        }
        TypeContext::InParameter => {
            format!(
                "global::System.Collections.Generic.Dictionary<{}, {}>",
                key_type_string,
                value_type_string,
            )
        }
        TypeContext::ReturnParameter => {
            format!(
                "global::System.Collections.Generic.IEnumerable<global::System.Collections.Generic.KeyValuePair<{}, {}>>",
                key_type_string,
                value_type_string,
            )
        }
    }
}
