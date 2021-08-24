// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;

use crate::code_block::CodeBlock;

pub fn encode_data_members(members: &[&Member], ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let (required_members, tagged_members) = get_sorted_members(members);

    let mut bit_sequence_index = -1;
    // Tagged members are encoded in a dictionary and don't count towards the optional bit sequence size.
    let bit_sequence_size = get_bit_sequence_size(members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = encoder.EncodeBitSequence({});",
            bit_sequence_size
        );
        bit_sequence_index = 0;
    }

    for member in required_members {
        // TODO: actually pass scope and param
        let encode_member = encode_type(
            &member.data_type,
            &mut bit_sequence_index,
            true,
            "scope",
            "param",
            ast,
        );
        code.writeln(&encode_member);
    }

    code
}

pub fn encode_type(
    type_ref: &TypeRef,
    bit_sequence_index: &mut i32,
    for_nested_type: bool,
    scope: &str,
    param: &str,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let node = type_ref.definition(ast);

    if type_ref.is_optional {
        match node {
            Node::Interface(_, _) => {
                writeln!(code, "encoder.EncodeNullableProxy({}?.Proxy);", param)
            }
            // Node::Class(_,_) //TODO: classes
            _ => {
                assert!(*bit_sequence_index > 0);
                let read_only_memory = if let Node::Sequence(_, sequence_def) = node {
                    let has_custom_type = sequence_def.element_type.has_attribute("cs:generic:");
                    sequence_def.is_element_fixed_sized_numeric(ast)
                        && !has_custom_type
                        && !for_nested_type
                } else {
                    false
                };
            }
        }
    } else {
    }

    code
}
