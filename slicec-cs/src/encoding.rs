// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;

use crate::code_block::CodeBlock;
use crate::cs_util::*;

pub fn encode_data_members(members: &[&Member], ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let (required_members, tagged_members) = get_sorted_members(members);

    let mut bit_sequence_index = -1;
    // Tagged members are encoded in a dictionary and don't count towards the optional bit sequence
    // size.
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

    // Encode tagged
    let mut current_tag = -1; // sanity check to ensure tags are sorted
    for member in tagged_members {
        let tag = member.tag.unwrap();
        assert!((tag as i32) > current_tag);
        current_tag = tag as i32;
        // TODO: tags are not yet supported
        code.writeln(&encode_tagged_type(member, tag, "scope", "param", ast));
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

    match node {
        Node::Interface(_, _) => {
            writeln!(code, "encoder.EncodeProxy({}.Proxy);", param)
        } // Node::Class(_, _) => {} //TODO
        Node::Primitive(_, _) => {
            writeln!(code, "encoder.Encode{}({});", builtin_suffix(node), param)
        }
        Node::Struct(_, _) => {
            writeln!(code, "{}.Encode(encoder);", param)
        }
        Node::Sequence(_, sequence_def) => code.writeln(&encode_sequence(
            sequence_def,
            scope,
            param,
            !for_nested_type,
            !for_nested_type,
            ast,
        )),
        Node::Dictionary(_, dictionary_def) => {
            code.writeln(&encode_dictionary(dictionary_def, scope, param, ast))
        }
        Node::Enum(_, enum_def) => {
            writeln!(
                code,
                "{helper}.Encode{name}(encoder, {param});",
                helper = helper_name(enum_def, scope),
                name = node.as_named_symbol().unwrap().identifier(),
                param = param
            );
        }
        _ => panic!("Node does not represent a type: {:?}", node),
    }

    if type_ref.is_optional {
        code = encode_as_optional(
            type_ref,
            bit_sequence_index,
            for_nested_type,
            scope,
            param,
            &code,
            ast,
        );
    }

    code
}

pub fn encode_tagged_type(
    member: &Member,
    tag: u32,
    scope: &str,
    param: &str,
    ast: &Ast,
) -> CodeBlock {
    "".into()
}

pub fn encode_sequence(
    sequence_def: &Sequence,
    scope: &str,
    value: &str,
    is_param: bool,
    is_read_only: bool,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let has_custom_type = false; // TODO: get from sequence metadata
    let mut args = Vec::new();

    if sequence_def.is_element_fixed_sized_numeric(ast) && (is_read_only && !has_custom_type) {
        if is_param && is_read_only && !has_custom_type {
            args.push(format!("{}.Span", value));
        } else {
            args.push(value.to_owned());
        }
    } else {
        args.push(value.to_owned());

        if sequence_def.element_type.encode_using_bit_sequence(ast)
            && is_reference_type(&sequence_def.element_type, ast)
        {
            assert!(sequence_def.element_type.is_optional);
            args.push("withBitSequence: true".to_owned());
        }

        args.push(
            encode_action(
                &sequence_def.element_type,
                scope,
                is_read_only,
                is_param,
                ast,
            )
            .to_string(),
        );
    }

    write!(
        code,
        "encoder.EncodeSequence({args})",
        args = args.join(", ")
    );

    code
}

pub fn encode_dictionary(
    dictionary_def: &Dictionary,
    scope: &str,
    param: &str,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let mut args = vec![param.to_owned()];

    let with_bit_sequence = dictionary_def.value_type.encode_using_bit_sequence(ast);

    if with_bit_sequence && is_reference_type(&dictionary_def.value_type, ast) {
        args.push("withBitSequence: true".to_owned());
    }
    args.push(encode_action(&dictionary_def.key_type, scope, false, false, ast).to_string());
    args.push(encode_action(&dictionary_def.value_type, scope, false, false, ast).to_string());

    write!(
        code,
        "encoder.EncodeDictionary({args})",
        args = args.join(", ")
    );

    code
}

pub fn encode_as_optional(
    type_ref: &TypeRef,
    bit_sequence_index: &mut i32,
    for_nested_type: bool,
    param: &str,
    scope: &str,
    encode_type: &CodeBlock,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();
    let node = type_ref.definition(ast);

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

            // A null T[]? or List<T>? is implicitly converted into a default aka null
            // ReadOnlyMemory<T> or ReadOnlySpan<T>. Furthermore, the span of a default
            // ReadOnlyMemory<T> is a default ReadOnlySpan<T>, which is distinct from
            // the span of an empty sequence. This is why the "value.Span != null" below
            // works correctly.
            writeln!(
                code,
                "\
if ({param}{as_span} != null)
{{
{encode_type}
}}
else
{{
bitSequence[{bit_sequence_index}] = false;
}}
",
                param = param,
                as_span = if read_only_memory { ".Span" } else { "" },
                encode_type = encode_type,
                bit_sequence_index = *bit_sequence_index
            );
            *bit_sequence_index += 1;
        }
    }

    code
}

pub fn encode_action(
    type_def: &TypeRef,
    scope: &str,
    is_read_only: bool,
    is_param: bool,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let node = type_def.definition(ast);

    if type_def.is_optional {
        match node {
            Node::Interface(_, _) => {
                write!(
                    code,
                    "(encoder, value) => encoder.EncodeNullableProxy(value?.Proxy)"
                )
            } // TODO: Node::Class (see C++ code)
            _ => panic!("expected interface or class"),
        }
    } else {
        match node {
            Node::Interface(_, _) => {
                write!(code, "(encoder, value) => encoder.EncodeProxy(value.Proxy)")
            } // TODO: Node::Class
            Node::Primitive(_, _) => {
                write!(
                    code,
                    "(encoder, value) => encoder.Encode{}(value)",
                    builtin_suffix(node)
                )
            }
            Node::Enum(_, enum_def) => {
                write!(
                    code,
                    "(encoder, value) => {helper}.Encode{name}(encoder, value)",
                    helper = helper_name(enum_def, scope),
                    name = enum_def.identifier()
                )
            }
            Node::Dictionary(_, dictionary_def) => {
                write!(
                    code,
                    "(encoder, dictionary) => {}",
                    encode_dictionary(dictionary_def, scope, "dictionary", ast)
                );
            }
            Node::Sequence(_, sequence_def) => {
                // We generate the sequence encoder inline, so this function must not be called when
                // the top-level object is not cached.
                write!(
                    code,
                    "(encoder, sequence) => {}",
                    encode_sequence(sequence_def, scope, "sequence", is_read_only, is_param, ast)
                )
            }
            Node::Struct(_, _) => {
                write!(code, "(encoder, value) => value.Encode(encoder)")
            }
            _ => panic!(""),
        }
    }

    code
}

pub fn encode_operation(operation: &Operation, return_type: bool, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();
    let ns = get_namespace(operation);

    let members = if return_type {
        operation.return_members(ast)
    } else {
        operation.parameters(ast)
    };

    let (members, streamed_members): (Vec<&Member>, Vec<&Member>) =
        members.iter().partition(|m| !m.data_type.is_streamed);

    let (required_members, tagged_members) = get_sorted_members(&members);

    let mut bit_sequence_index = -1;
    let bit_sequence_size = get_bit_sequence_size(&members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = encoder.EncodeBitSequence({})",
            bit_sequence_size
        );
        bit_sequence_index = 0;
    }

    for member in required_members {
        let param = if members.len() == 1 {
            "value".to_owned()
        } else {
            "value.{}".to_owned() + &field_name(member, "")
        };
        let encode_member = encode_type(
            &member.data_type,
            &mut bit_sequence_index,
            true,
            &ns,
            &param,
            ast,
        );
        code.writeln(&encode_member);
    }

    if bit_sequence_size > 0 {
        assert_eq!(bit_sequence_index, bit_sequence_size);
    }

    for member in tagged_members {
        let tag = member.tag.unwrap();
        // TODO: tags are not yet supported
        code.writeln(&encode_tagged_type(member, tag, "scope", "param", ast));
    }

    code
}
