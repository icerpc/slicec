// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::Primitive;

pub fn type_to_string(node: &Node, ast: &Ast) -> String {
    match node {
        Node::Struct(_, struct_def) => {
            let mut identifier = struct_def.scope.as_ref().unwrap().clone() + "::" + struct_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        }
        Node::Interface(_, interface_def) => {
            let mut identifier = interface_def.scope.as_ref().unwrap().clone() + "::" + interface_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        }
        Node::Sequence(_, sequence) => {
            let element_type = ast.resolve_index(sequence.element_type.definition.unwrap());
            let element_type_string = type_to_string(element_type, ast);
            "global::System.Collections.Generic.IList<".to_owned() + &element_type_string + ">"
        }
        Node::Dictionary(_, dictionary) => {
            let key_type = ast.resolve_index(dictionary.key_type.definition.unwrap());
            let value_type = ast.resolve_index(dictionary.value_type.definition.unwrap());
            let key_type_string = type_to_string(key_type, ast);
            let value_type_string = type_to_string(value_type, ast);
            "global::System.Collections.Generic.IDictionary<".to_owned() + &key_type_string + ", " + &value_type_string + ">"
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
            panic!("Node does not represent a type:{:?}", node);
        },
    }
}
