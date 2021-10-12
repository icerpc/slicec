// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;

use crate::code_block::CodeBlock;
use crate::cs_util::*;

pub fn encode_data_members(
    members: &[&Member],
    scope: &str,
    field_type: FieldType,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let (required_members, tagged_members) = get_sorted_members(members);

    let mut bit_sequence_index = -1;
    // Tagged members are encoded in a dictionary and don't count towards the optional bit sequence
    // size.
    let bit_sequence_size = get_bit_sequence_size(&required_members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = encoder.EncodeBitSequence({});",
            bit_sequence_size
        );
        bit_sequence_index = 0;
    }

    for member in required_members {
        let param = format!("this.{}", field_name(member, field_type));

        let encode_member = encode_type(
            &member.data_type,
            &mut bit_sequence_index,
            true,
            scope,
            &param,
            ast,
        );
        code.writeln(&encode_member);
    }

    // Encode tagged
    for member in tagged_members {
        let param = format!("this.{}", field_name(member, field_type));
        code.writeln(&encode_tagged_type(member, scope, &param, true, ast));
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
        }
        Node::Class(_, _) => {
            writeln!(code, "encoder.EncodeClass({});", param)
        }
        Node::Primitive(_, _) => {
            writeln!(code, "encoder.Encode{}({});", builtin_suffix(node), param)
        }
        Node::Struct(_, _) => {
            writeln!(code, "{}.Encode(encoder);", param)
        }
        Node::Sequence(_, sequence_def) => writeln!(
            code,
            "{};",
            encode_sequence(
                sequence_def,
                scope,
                param,
                !for_nested_type,
                !for_nested_type,
                ast,
            ),
        ),
        Node::Dictionary(_, dictionary_def) => {
            writeln!(
                code,
                "{};",
                encode_dictionary(dictionary_def, scope, param, ast)
            )
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
            param,
            &code,
            ast,
        );
    }

    code
}

// TODO: should is_data_member be TypeContext instead of bool?
pub fn encode_tagged_type(
    member: &Member,
    scope: &str,
    param: &str,
    is_data_member: bool,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    assert!(member.data_type.is_optional && member.tag.is_some());

    let tag = member.tag.unwrap();

    let node = member.data_type.definition(ast);

    let read_only_memory = match node {
        Node::Sequence(_, sequence_def)
            if sequence_def.is_fixed_size(ast)
                && !is_data_member
                && !member.data_type.has_attribute("cs:generic") =>
        {
            true
        }
        _ => false,
    };

    let value = if is_value_type(&member.data_type, ast) && !read_only_memory {
        format!("{}.Value", param)
    } else {
        param.to_owned()
    };

    // For types with a known size, we provide a size parameter with the size of the tagged
    // param/member:
    let mut size_parameter = String::new();

    match node {
        Node::Primitive(_, primitive_def) => {
            if primitive_def.is_fixed_size(ast) {
                size_parameter = primitive_def.min_wire_size(ast).to_string();
            } else if !matches!(primitive_def, Primitive::String) {
                if primitive_def.is_unsigned_numeric() {
                    size_parameter = format!("IceRpc.GetVarULongEncodedSize({})", value)
                } else {
                    size_parameter = format!("IceRpc.GetVarLongEncodedSize({})", value)
                }
            }
            // else no size
        }
        Node::Struct(_, struct_def) => {
            if struct_def.is_fixed_size(ast) {
                size_parameter = struct_def.min_wire_size(ast).to_string();
            }
        }
        Node::Enum(_, enum_def) => {
            if let Some(underlying) = &enum_def.underlying {
                size_parameter = underlying.min_wire_size(ast).to_string();
            } else {
                size_parameter = format!("encoder.GetSizeLength((int){})", value);
            }
        }
        Node::Sequence(_, sequence_def) => {
            let element_type = &sequence_def.element_type;

            if element_type.is_fixed_size(ast) {
                if read_only_memory {
                    size_parameter = format!(
                        "encoder.GetSizeLength({value}) + {element_min_wire_size} * {value}.Length",
                        value = value,
                        element_min_wire_size = element_type.min_wire_size(ast)
                    );
                } else {
                    writeln!(code, "int count = {}.Count();", value);
                    size_parameter = format!(
                        "encoder.GetSizeLength(count) + {} * count",
                        element_type.min_wire_size(ast)
                    )
                }
            }
        }
        Node::Dictionary(_, dictionary_def) => {
            let key_type = &dictionary_def.key_type;
            let value_type = &dictionary_def.value_type;

            if key_type.is_fixed_size(ast) && value_type.is_fixed_size(ast) {
                writeln!(code, "int count = {}.Count();", value);
                size_parameter = format!(
                    "encoder.GetSizeLength(count) + {min_wire_size} * count",
                    min_wire_size = key_type.min_wire_size(ast) + value_type.min_wire_size(ast)
                );
            }
        }
        Node::Interface(_, _) => {}
        _ => panic!("unexpected node type: {:?}", node),
    }

    let mut args = vec![];
    args.push(tag.to_string());

    args.push(format!(
        "IceRpc.Slice.TagFormat.{}",
        member.data_type.tag_format(ast)
    ));
    args.push(value);
    if !size_parameter.is_empty() {
        args.push("size: ".to_owned() + &size_parameter);
    }
    args.push(
        encode_action(
            &member.data_type,
            scope,
            !is_data_member,
            !is_data_member,
            ast,
        )
        .to_string(),
    );

    writeln!(
        code,
        "\
if ({param} != null)
{{
    encoder.EncodeTagged({args});
}}",
        param = if read_only_memory {
            param.to_owned() + ".Span"
        } else {
            param.to_owned()
        },
        args = args.join(", "),
    );

    code
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
    encode_type: &CodeBlock,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();
    let node = type_ref.definition(ast);

    match node {
        Node::Interface(_, _) => {
            writeln!(code, "encoder.EncodeNullableProxy({}?.Proxy);", param)
        }
        Node::Class(_, _) => {
            writeln!(code, "encoder.EncodeNullableClass({});", param)
        }
        _ => {
            assert!(*bit_sequence_index >= 0);
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

    let is_optional = type_def.is_optional;

    let value = if type_def.is_optional {
        if is_value_type(type_def, ast) {
            "value!.Value"
        } else {
            "value!"
        }
    } else {
        "value"
    };

    match node {
        Node::Interface(_, _) => {
            if is_optional {
                write!(
                    code,
                    "(encoder, value) => encoder.EncodeNullableProxy(value?.Proxy)"
                );
            } else {
                write!(code, "(encoder, value) => encoder.EncodeProxy(value.Proxy)");
            }
        }
        Node::Class(_, _) => {
            if is_optional {
                write!(
                    code,
                    "(encoder, value) => encoder.EncodeNullableClass(value)"
                );
            } else {
                write!(code, "(encoder, value) => encoder.EncodeClass(value)");
            }
        }
        Node::Primitive(_, _) => {
            write!(
                code,
                "(encoder, value) => encoder.Encode{builtin_type}({value})",
                builtin_type = builtin_suffix(node),
                value = value
            )
        }
        Node::Enum(_, enum_def) => {
            write!(
                code,
                "(encoder, value) => {helper}.Encode{name}(encoder, {value})",
                helper = helper_name(enum_def, scope),
                name = enum_def.identifier(),
                value = value
            )
        }
        Node::Dictionary(_, dictionary_def) => {
            write!(
                code,
                "(encoder, value) => {}",
                encode_dictionary(dictionary_def, scope, "dictionary", ast)
            );
        }
        Node::Sequence(_, sequence_def) => {
            // We generate the sequence encoder inline, so this function must not be called when
            // the top-level object is not cached.
            write!(
                code,
                "(encoder, value) => {}",
                encode_sequence(sequence_def, scope, "sequence", is_read_only, is_param, ast)
            )
        }
        Node::Struct(_, _) => {
            write!(code, "(encoder, value) => value.Encode(encoder)")
        }
        _ => panic!("Unexpected node type {:?}", node),
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

    let (members, _streamed_members): (Vec<&Member>, Vec<&Member>) =
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
            "value.".to_owned() + &field_name(member, FieldType::NonMangled)
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
        let param = if members.len() == 1 {
            "value".to_owned()
        } else {
            "value.".to_owned() + &field_name(member, FieldType::NonMangled)
        };
        code.writeln(&encode_tagged_type(member, &ns, &param, false, ast));
    }

    code
}
