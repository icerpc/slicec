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
    let bit_sequence_size = get_bit_sequence_size(members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = decoder.DecodeBitSequence({});",
            bit_sequence_size,
        );

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

// TODO: scope and param (scope should be passed in to type_to_string)
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

    write!(code, "{} = ", param);

    if type_ref.is_optional {
        match node {
            Node::Interface(_, _) => {
                // does not use bit sequence
                writeln!(
                    code,
                    "IceRpc.IceDecoderPrxExtensions.DecodeNullablePrx<{}>(decoder);",
                    type_string
                );
                return code;
            }
            // TODO: this else if once we have Node::Class
            // Node::Class(_, class_def) => {
            // // does not use bit sequence
            // write!(
            //     "decoder.DecodeNullableClass<{}>();\n",
            //     type_to_string(
            //         astresolve_index(type_ref.definition.unwrap()),
            //         ast,
            //         TypeContext::Incoming
            //     ));
            //return code;
            // }
            _ => {
                assert!(*bit_sequence_index > 0);
                write!(code, "bitSequence[{}]", *bit_sequence_index);
                *bit_sequence_index += 1;
                // keep going
            }
        }
    }

    match node {
        Node::Interface(_, _) => {
            assert!(!type_ref.is_optional);
            write!(code, "new {}(decoder.DecodeProxy());", type_string)
        }
        // Node::Class(_, class_def) => {} // TODO: Class not yet implemented in the ast
        Node::Primitive(_, primitive_def) => {
            write!(
                code,
                "decoder.Decode{}()",
                primitive_type_suffix(primitive_def),
            );
        }
        Node::Struct(_, _) => {
            write!(
                code,
                "new {}(decoder)",
                get_scoped_unqualified(node, scope, ast),
            );
        }
        Node::Dictionary(_, dictionary) => code.write(&decode_dictionary(dictionary, scope, ast)),
        Node::Sequence(_, sequence) => code.write(&decode_sequence(sequence, scope, ast)),
        _ => {
            write!(
                code,
                "{}.Decode{}(decoder)",
                helper_name(type_ref, scope, ast),
                type_string,
            );
        }
    }

    if type_ref.is_optional {
        code.write(" : null");
    }

    code.write(";");

    code
}

pub fn decode_dictionary(dictionary: &Dictionary, scope: &str, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();
    code.writeln("//TODO");
    code
}

pub fn decode_sequence(sequence: &Sequence, scope: &str, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    //TOOD: check for generic "cs:generic:" attribute
    // let generic = sequence.element_type.
    let generic_attribute: Option<&str> = None; // TODO: temporary
    let element_type = &sequence.element_type;
    let element_node = element_type.definition(ast);

    if let Some(generic_attribute) = generic_attribute {
        let mut args: String;

        args = "".to_owned();
        match element_node {
            Node::Primitive(_, primitive)
                if primitive.is_numeric_or_bool() && primitive.is_fixed_size(ast) =>
            {
                //TODO type_to_string should use the scope: c++ => typeToString(type, scope)
                // We always read an array even when mapped to a collection, as it's expected to be faster than unmarshaling
                // the collection elements one by one.
                args = format!(
                    "decoder.DecodeArray<{}>()",
                    type_to_string(element_node, ast, TypeContext::Incoming)
                );
            }
            Node::Enum(_, enum_def) if enum_def.underlying.is_some() && enum_def.is_unchecked => {
                //TODO type_to_string should use the scope: c++ => typeToString(type, scope)
                // We always read an array even when mapped to a collection, as it's expected to be faster than unmarshaling
                // the collection elements one by one.
                args = format!(
                    "decoder.DecodeArray<{}>()",
                    type_to_string(element_node, ast, TypeContext::Incoming)
                );
            }
            Node::Enum(_, enum_def) if enum_def.underlying.is_some() => {
                //TODO both type_to_string  should use the scope: c++ => typeToString(enum, scope)
                let underlying_type = enum_def.underlying_type(ast);
                args = format!(
                    "decoder.DecodeArray(({enum_type_name} e) => _ = {helper}.As{name}(({underlying_type})e))",
                    enum_type_name =type_to_string(element_node, ast, TypeContext::Incoming),
                    helper = helper_name(element_type, scope, ast),
                    name = enum_def.identifier(),
                    underlying_type = underlying_type.as_named_symbol().unwrap().identifier(),
                );
            }
            _ => {
                if element_type.is_optional && element_type.encode_using_bit_sequence(ast) {
                    args = format!(
                        "decoder.DecodeSequence({}{})",
                        if !is_value_type(element_type, ast) {
                            "withBitSequence: true, "
                        } else {
                            ""
                        },
                        decode_func(element_type, scope, ast)
                    );
                } else {
                    args = format!(
                        "decoder.DecodeSequence(minElementSize: {}, {})",
                        element_type.min_wire_size(ast),
                        decode_func(element_type, scope, ast)
                    );
                }
            }
        }

        if generic_attribute == "Stack" {
            args = format!("(global::System.Linq.Enumerable.Reverse{})", args);
        }

        //TODO type_to_string should use the scope: c++ =>  typeToString(seq, scope, false, true)
        write!(
            code,
            "new {}({})",
            type_to_string(element_node, ast, TypeContext::Incoming),
            args
        );
    } else {
        // generic arg for the decoder
        let generic_arg: String;
        // the args for DecodeArray()
        let decoder_args: String;

        match element_node {
            Node::Primitive(_, primitive)
                if (primitive.is_numeric_or_bool() && primitive.is_fixed_size(ast)) =>
            {
                //TODO: use the scope (see c++ code below)
                generic_arg = type_to_string(element_node, ast, TypeContext::Incoming);
                decoder_args = "".to_owned();
                // out << "decoder.DecodeArray<" << typeToString(type, scope) << ">()";
            }
            Node::Enum(_, enum_def) if (enum_def.underlying.is_some() && enum_def.is_unchecked) => {
                //TODO: use the scope (see c++ code below)
                generic_arg = type_to_string(element_node, ast, TypeContext::Incoming);
                decoder_args = "".to_owned();
                // out << "decoder.DecodeArray<" << typeToString(type, scope) << ">()";
            }
            Node::Enum(_, enum_def) if (enum_def.underlying.is_some()) => {
                //TODO
                generic_arg = "".to_owned();
                decoder_args = "".to_owned();

                // out << "decoder.DecodeArray((" << typeToString(en, scope) << " e) => _ = " << helperName(en, scope)
                // << ".As" << en->name() << "((" << typeToString(en->underlying(), scope) << ")e))";
            }
            _ => {
                generic_arg = "".to_owned();
                if element_type.is_optional && element_type.encode_using_bit_sequence(ast) {
                    decoder_args = format!(
                        "{}{}",
                        if !is_value_type(element_type, ast) {
                            "withBitSequence: true, "
                        } else {
                            ""
                        },
                        decode_func(element_type, scope, ast)
                    );
                } else {
                    decoder_args = format!(
                        "minElementSize:{}, {}",
                        element_type.min_wire_size(ast),
                        decode_func(element_type, scope, ast)
                    );
                }
            }
        }

        write!(
            code,
            "decoder.DecodeArray{generic_arg}({args})",
            generic_arg = if generic_arg.is_empty() {
                "".to_owned()
            } else {
                format!("<{}>", generic_arg)
            },
            args = decoder_args,
        )
    }

    code
}

pub fn decode_func(type_ref: &TypeRef, scope: &str, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();
    let node = type_ref.definition(ast);

    if type_ref.is_optional {
        match node {
            Node::Interface(_, _) => {
                write!(
                    code,
                    "decoder => IceRpc.IceDecoderPrxExtensions.DecodeNullablePrx<{}>(decoder)",
                    type_to_string(node, ast, TypeContext::Incoming)
                );
            }
            //TODO Node::Class(_, _)
            _ => panic!("Node must be either an interface or class"),
        }
    } else {
        match node {
            // Node::Class(_, ) => {} // TODO when we have class support
            Node::Interface(_, interface_def) => {
                //TODO: type_to_string should take the scope
                write!(
                    code,
                    "decoder = new {}(decoder.DecodeProxy())",
                    type_to_string(node, ast, TypeContext::Incoming)
                );
            }
            //TODO review logic here wrt Builtin && usesClasses() (see c++ code)
            Node::Primitive(_, _) => {
                write!(code, "decoder => decoder.Decode{}()", builtin_suffix(node));
            }
            Node::Sequence(_, sequence) => {
                write!(code, "decoder => {}", decode_sequence(sequence, scope, ast));
            }
            Node::Dictionary(_, dictionary) => {
                write!(
                    code,
                    "decoder => {}",
                    decode_dictionary(dictionary, scope, ast)
                );
            }
            Node::Enum(_, enum_def) => {
                write!(
                    code,
                    "decoder => {}.Decode{}(decoder)",
                    helper_name(type_ref, scope, ast),
                    enum_def.identifier()
                );
            }
            Node::Struct(_, _) => {
                write!(
                    code,
                    "decoder => new {}(decoder)",
                    //TODO: scope
                    type_to_string(node, ast, TypeContext::Incoming)
                );
            }
            _ => panic!("unexpected node type"),
        }
    }

    code
}

//TODO move to utils
pub fn is_value_type(type_ref: &TypeRef, ast: &Ast) -> bool {
    match type_ref.definition(ast) {
        Node::Primitive(_, primitive) => !matches!(primitive, Primitive::String),
        Node::Enum(_, _) | Node::Struct(_, _) | Node::Interface(_, _) => true,
        _ => false,
    }
}
