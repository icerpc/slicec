// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_block::CodeBlock;
use crate::cs_util::*;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;

pub fn decode_data_members(members: &[&Member], ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let (required_members, tagged_members) = get_sorted_members(members);

    let mut bit_sequence_index = -1;
    let bit_sequence_size = get_bit_sequence_size(&members, ast);

    if bit_sequence_size > 0 {
        code.writeln(&format!(
            "var bitSequence = decoder.DecodeBitSequence({});",
            bit_sequence_size
        ));

        bit_sequence_index = 0;
    }

    // Encode required members
    for member in required_members {
        let decode_member = decode_type(
            &member.data_type,
            &mut bit_sequence_index,
            "scope",
            // "this." + fixId(fieldName(member), baseTypes) //TODO: port this from C++ for param
            "param",
            ast,
        );

        code.writeln(&decode_member);
    }

    // Encode tagged members
    let mut current_tag = -1; // sanity check to ensure tags are sorted
    for member in tagged_members {
        let tag = member.tag.unwrap();
        assert!(tag > current_tag);
        current_tag = tag;

        // decode_tagged_type()
    }

    if bit_sequence_size > 0 {
        assert_eq!(bit_sequence_index, bit_sequence_size);
    }

    code
}

//TODO: scope and param (scope should be passed in to type_to_string)
pub fn decode_type(
    type_ref: &TypeRef,
    bit_sequence_index: &mut i32,
    scope: &str,
    param: &str,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let node = ast.resolve_index(type_ref.definition.unwrap());
    let type_string = type_to_string(node, ast, TypeContext::Incoming); // TODO: the scope

    code.write(&format!("{} = ", param));

    if type_ref.is_optional {
        match node {
            Node::Interface(_, _) => {
                // does not use bit sequence
                code.writeln(&format!(
                    "IceRpc.IceDecoderPrxExtensions.DecodeNullablePrx<{}>(decoder);",
                    type_string
                ));
                return code;
            }
            //TODO: this else if once we have Node::Class
            // Node::Class(_, class_def) => {
            // // does not use bit sequence
            // code.writeln(&format!(
            //     "decoder.DecodeNullableClass<{}>();",
            //     type_to_string(
            //         ast.resolve_index(type_ref.definition.unwrap()),
            //         ast,
            //         TypeContext::Incoming
            //     )
            //return code;
            // ));
            // }
            _ => {
                assert!(*bit_sequence_index > 0);
                code.write(&format!("bitSequence[{}]", *bit_sequence_index));
                *bit_sequence_index += 1;
                // keep going
            }
        }
    }

    match node {
        Node::Interface(_, _) => {
            assert!(!type_ref.is_optional);
            code.write(&format!("new {}(decoder.DecodeProxy());", type_string));
        }
        // Node::Class(_, class_def) => {} // TODO: Class not yet implemented in the ast
        Node::Primitive(_, primitive_def) => code.write(&format!(
            "decoder.Decode{}()",
            primitive_type_suffix(primitive_def)
        )),
        Node::Struct(_, _) => code.write(&format!(
            "new {}(decoder)",
            get_scoped_unqualified(node, scope, ast)
        )),
        Node::Dictionary(_, _) => {}
        Node::Sequence(_, _) => {}
        _ => {
            code.write(&format!(
                "{}.Decode{}(decoder)",
                helper_name(type_ref, scope, ast),
                type_string
            ));
            // out << helperName(underlying, scope) << ".Decode" << contained->name() << "(decoder)";
        }
    }

    if type_ref.is_optional {
        code.write(" : null");
    }

    code.write(";");

    code
}
