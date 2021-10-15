// Copyright (c) ZeroC, Inc. All rights reserved.
use crate::code_block::CodeBlock;
use crate::cs_util::*;
use crate::traits::*;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;

pub fn decode_data_members(
    members: &[&Member],
    scope: &str,
    field_type: FieldType,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let (required_members, tagged_members) = get_sorted_members(members);

    let mut bit_sequence_index = -1;
    let bit_sequence_size = get_bit_sequence_size(members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = decoder.DecodeBitSequence({});",
            bit_sequence_size
        );
        bit_sequence_index = 0;
    }

    // Decode required members
    for member in required_members {
        let param = format!("this.{}", member.field_name(field_type));
        let decode_member = decode_member(member, &mut bit_sequence_index, scope, &param, ast);
        code.writeln(&decode_member);
    }

    // Decode tagged members

    for member in tagged_members {
        let param = format!("this.{}", member.field_name(field_type));
        code.writeln(&decode_tagged_member(member, scope, &param, ast));
    }

    if bit_sequence_size > 0 {
        assert_eq!(bit_sequence_index, bit_sequence_size);
    }

    code
}

// TODO: scope and param (scope should be passed in to type_to_string)
pub fn decode_member(
    member: &Member,
    bit_sequence_index: &mut i32,
    scope: &str,
    param: &str,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();
    let data_type = &member.data_type;

    let node = data_type.definition(ast);
    let type_string = data_type.type_to_string(scope, ast, TypeContext::Incoming);

    write!(code, "{} = ", param);

    if data_type.is_optional {
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
            Node::Class(_, _) => {
                // does not use bit sequence
                writeln!(code, "decoder.DecodeNullableClass<{}>();", type_string);
                return code;
            }
            _ => {
                assert!(*bit_sequence_index >= 0);
                write!(code, "bitSequence[{}] ? ", *bit_sequence_index);
                *bit_sequence_index += 1;
                // keep going
            }
        }
    }

    match node {
        Node::Interface(_, _) => {
            assert!(!data_type.is_optional);
            write!(code, "new {}(decoder.DecodeProxy());", type_string);
        }
        Node::Class(_, _) => {
            assert!(!data_type.is_optional);
            write!(code, "decoder.DecodeClass<{}>();", type_string);
        }
        Node::Primitive(_, primitive_def) => {
            write!(code, "decoder.Decode{}()", primitive_def.type_suffix());
        }
        Node::Struct(_, struct_def) => {
            write!(
                code,
                "new {}(decoder)",
                struct_def.escape_scoped_identifier(CaseStyle::Pascal, scope),
            );
        }
        Node::Dictionary(_, dictionary) => code.write(&decode_dictionary(dictionary, scope, ast)),
        Node::Sequence(_, sequence) => {
            code.write(&decode_sequence(data_type, sequence, scope, ast))
        }
        Node::Enum(_, enum_def) => {
            write!(
                code,
                "{}.Decode{}(decoder)",
                enum_def.helper_name(scope),
                type_string,
            );
        }
        _ => panic!("Node does not represent a type: {:?}", node),
    }

    if data_type.is_optional {
        code.write(" : null");
    }

    code.write(";");

    code
}

pub fn decode_tagged_member(member: &Member, scope: &str, param: &str, ast: &Ast) -> CodeBlock {
    assert!(member.data_type.is_optional && member.tag.is_some());

    let tag = member.tag.unwrap();

    format!(
        "{param} = decoder.DecodeType({tag}, IceRpc.Slice.TagFormat.{tag_format}, {decode_func});",
        param = param,
        tag = tag,
        tag_format = member.data_type.tag_format(ast),
        decode_func = decode_func(&member.data_type, scope, ast)
    )
    .into()
}

pub fn decode_dictionary(dictionary_def: &Dictionary, scope: &str, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    // TOOD: check for generic "cs:generic:" attribute
    // let generic = sequence.element_type.
    let value_type = &dictionary_def.value_type;
    let value_node = value_type.definition(ast);
    let generic_attribute: Option<&str> = None; // TODO: temporary

    let with_bit_sequence = value_type.encode_using_bit_sequence(ast);

    let method = match generic_attribute {
        Some(_) => "DecodeSortedDictionary",
        None => "DecodeDictionary",
    };

    let mut args = vec![format!("minKeySize: {}", dictionary_def.key_type.min_wire_size(ast))];

    if !with_bit_sequence {
        args.push(format!("minValueSize: {}", value_type.min_wire_size(ast)));
    }

    if with_bit_sequence && value_type.is_reference_type(ast) {
        args.push("withBitSequence: true".to_owned());
    }

    // decode key
    args.push(decode_func(&dictionary_def.key_type, scope, ast).to_string());

    // decode value
    let mut decode_value = decode_func(value_type, scope, ast);
    match value_node {
        Node::Sequence(_, _) | Node::Dictionary(_, _) => {
            write!(
                decode_value,
                " as {}",
                value_type.type_to_string(scope, ast, TypeContext::Incoming)
            );
        }
        _ => {}
    }

    write!(
        code,
        "decoder.{method}({args})",
        method = method,
        args = args.join(", ")
    );
    code
}

pub fn decode_sequence(
    type_ref: &TypeRef,
    sequence: &Sequence,
    scope: &str,
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();
    let element_type = &sequence.element_type;
    let element_node = element_type.definition(ast);

    if let Some(generic_attribute) = type_ref.find_attribute("cs:generic") {
        let args: String;
        assert!(!generic_attribute.is_empty());

        match element_node {
            Node::Primitive(_, primitive)
                if primitive.is_numeric_or_bool() && primitive.is_fixed_size(ast) =>
            {
                // We always read an array even when mapped to a collection, as it's expected to be
                // faster than unmarshaling the collection elements one by one.
                args = format!(
                    "decoder.DecodeArray<{}>()",
                    element_type.type_to_string(scope, ast, TypeContext::Incoming)
                );
            }
            Node::Enum(_, enum_def) if enum_def.underlying.is_some() && enum_def.is_unchecked => {
                // We always read an array even when mapped to a collection, as it's expected to be
                // faster than unmarshaling the collection elements one by one.
                args = format!(
                    "decoder.DecodeArray<{}>()",
                    element_type.type_to_string(scope, ast, TypeContext::Incoming)
                );
            }
            Node::Enum(_, enum_def) if enum_def.underlying.is_some() => {
                let underlying_type = enum_def.underlying.as_ref().unwrap().definition(ast);
                args = format!(
                    "decoder.DecodeArray(({enum_type_name} e) => _ = {helper}.As{name}(({underlying_type})e))",
                    enum_type_name = element_type.type_to_string( scope, ast, TypeContext::Incoming),
                    helper = enum_def.helper_name(scope),
                    name = enum_def.identifier(),
                    underlying_type = underlying_type.as_named_symbol().unwrap().identifier(),
                );
            }
            _ => {
                if element_type.is_optional && element_type.encode_using_bit_sequence(ast) {
                    args = format!(
                        "decoder.DecodeSequence({}{})",
                        if element_type.is_reference_type(ast) {
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

        write!(
            code,
            "new {}({})",
            type_ref.type_to_string(scope, ast, TypeContext::Incoming),
            match generic_attribute.first().unwrap().as_str() {
                "Stack" => format!("global::System.Linq.Enumerable.Reverse({})", args),
                _ => args,
            }
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
                generic_arg = element_type.type_to_string(scope, ast, TypeContext::Incoming);
                decoder_args = "".to_owned();
            }
            Node::Enum(_, enum_def) if (enum_def.underlying.is_some() && enum_def.is_unchecked) => {
                generic_arg = element_type.type_to_string(scope, ast, TypeContext::Incoming);
                decoder_args = "".to_owned();
            }
            Node::Enum(_, enum_def) if (enum_def.underlying.is_some()) => {
                let underlying_type = enum_def.underlying.as_ref().unwrap().definition(ast);
                generic_arg = "".to_owned();
                decoder_args = format!("decoder.DecodeArray(({enum_type} e) => _ = {helper}.As{name}(({underlying_type})e))",
                                enum_type = element_type.type_to_string( scope, ast, TypeContext::Incoming),
                                helper = enum_def.helper_name(scope),
                                name = enum_def.identifier(),
                                underlying_type = underlying_type.as_named_symbol().unwrap().identifier());
            }
            _ => {
                generic_arg = "".to_owned();
                if element_type.is_optional && element_type.encode_using_bit_sequence(ast) {
                    decoder_args = format!(
                        "{}{}",
                        if element_type.is_reference_type(ast) {
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

    let is_optional = type_ref.is_optional;
    let name = type_ref.type_to_string(scope, ast, TypeContext::Incoming);

    match node {
        Node::Interface(_, _) => {
            if is_optional {
                write!(code, "decoder => decoder.DecodeNullablePrx<{}>()", name);
            } else {
                write!(code, "decoder => new {}(decoder.DecodeProxy())", name);
            }
        }
        Node::Class(_, _) => {
            if is_optional {
                write!(code, "decoder => decoder.DecodeNullableClass<{}>()", name);
            } else {
                write!(code, "decoder => decoder.DecodeClass<{}>()", name);
            }
        }
        Node::Primitive(_, primitive) => {
            write!(
                code,
                "decoder => decoder.Decode{}()",
                primitive.type_suffix()
            );
        }
        Node::Sequence(_, sequence) => {
            write!(
                code,
                "decoder => {}",
                decode_sequence(type_ref, sequence, scope, ast)
            );
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
                enum_def.helper_name(scope),
                enum_def.identifier()
            );
        }
        Node::Struct(_, _) => {
            write!(code, "decoder => new {}(decoder)", name);
        }
        _ => panic!("unexpected node type"),
    }

    code
}

pub fn decode_operation(operation: &Operation, return_type: bool, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let namespace = &operation.namespace();

    let (all_members, non_streamed_members) = if return_type {
        (
            operation.return_members(ast),
            operation.non_streamed_returns(ast),
        )
    } else {
        (
            operation.parameters(ast),
            operation.non_streamed_params(ast),
        )
    };

    let stream_member = if return_type {
        operation.stream_return(ast)
    } else {
        operation.stream_parameter(ast)
    };

    let (required_members, tagged_members) = get_sorted_members(&non_streamed_members);

    let mut bit_sequence_index = -1;
    let bit_sequence_size = get_bit_sequence_size(&non_streamed_members, ast);

    if bit_sequence_size > 0 {
        writeln!(
            code,
            "var bitSequence = decoder.DecodeBitSequence({});",
            bit_sequence_size
        );
        bit_sequence_index = 0;
    }

    for member in required_members {
        let decode_member = decode_member(
            member,
            &mut bit_sequence_index,
            namespace,
            &member.as_parameter_name("iceP_", true),
            ast,
        );

        writeln!(
            code,
            "{param_type} {decode}",
            param_type = member
                .data_type
                .type_to_string(namespace, ast, TypeContext::Incoming),
            decode = decode_member
        )
    }

    if bit_sequence_size > 0 {
        assert_eq!(bit_sequence_index, bit_sequence_size);
    }

    for member in tagged_members {
        let decode_member = decode_tagged_member(
            member,
            namespace,
            &member.as_parameter_name("iceP_", true),
            ast,
        );

        writeln!(
            code,
            "{param_type} {decode}",
            param_type = member
                .data_type
                .type_to_string(namespace, ast, TypeContext::Incoming),
            decode = decode_member
        )
    }

    if let Some(stream_member) = stream_member {
        let stream_param_type =
            stream_member
                .data_type
                .type_to_string(namespace, ast, TypeContext::Incoming);

        writeln!(
            code,
            "{param_type} {param_name}",
            param_type =
                stream_member
                    .data_type
                    .type_to_string(namespace, ast, TypeContext::Incoming),
            param_name = stream_member.as_parameter_name("iceP_", true)
        );

        let mut create_stream_param: CodeBlock = match stream_member.data_type.definition(ast) {
            Node::Primitive(_, primitive) if matches!(primitive, Primitive::Byte) => {
                if return_type {
                    "streamParamReceiver!.ToByteStream();".into()
                } else {
                    "IceRpc.StreamParamReceiver.ToByteStream(dispatch);".into()
                }
            }
            _ => {
                // TODO: is this if backwards? (copied from C++)
                if return_type {
                    format!(
                        "\
streamParamReceiver!.ToAsyncEnumerable<{stream_param_type}>(
    connection,
    invoker,
    payloadEncoding,
    {decode_func});",
                        stream_param_type = stream_param_type,
                        decode_func = decode_func(&stream_member.data_type, namespace, ast)
                    )
                    .into()
                } else {
                    format!(
                        "\
IceRpc.StreamParamReceiver.ToAsyncEnumerable<{stream_param_type}>(
    dispatch,
    {decode_func});",
                        stream_param_type = stream_param_type,
                        decode_func = decode_func(&stream_member.data_type, namespace, ast)
                    )
                    .into()
                }
            }
        };

        writeln!(
            code,
            "{param_type} {param_name} = {create_stream_param}",
            param_type = stream_param_type,
            param_name = stream_member.as_parameter_name("iceP_", true),
            create_stream_param = create_stream_param.indent()
        );
    }

    writeln!(code, "return {}", all_members.to_argument_tuple("iceP_"));

    code
}
