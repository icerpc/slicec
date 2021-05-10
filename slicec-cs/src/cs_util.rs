// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Node;
use slice::grammar::Builtin;

pub fn type_to_string(node: &Node) -> String {
    match node {
        Node::Struct(_, struct_def) => {
            let mut identifier = struct_def.scope.as_ref().unwrap().clone() + "::" + struct_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        },
        Node::Interface(_, interface_def) => {
            let mut identifier = interface_def.scope.as_ref().unwrap().clone() + "::" + interface_def.identifier();
            identifier.drain(2..).collect::<String>().replace("::", ".")
        },
        Node::Builtin(_, builtin) => {
            match builtin {
                Builtin::Int => { "int" },
                Builtin::String => { "string" },
            }.to_owned()
        },
        _ => {
            panic!("Node does not represent a type:{:?}", node);
        },
    }
}
