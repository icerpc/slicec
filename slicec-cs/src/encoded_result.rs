// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::grammar::Operation;
use slice::util::{CaseStyle, TypeContext};

use crate::builders::{CommentBuilder, ContainerBuilder, FunctionBuilder, FunctionType};
use crate::code_block::CodeBlock;
use crate::member_util::escape_parameter_name;
use crate::traits::*;

use crate::dispatch_visitor::response_encode_action;

pub fn encoded_result_struct(operation: &Operation, ast: &Ast) -> CodeBlock {
    let operation_name = operation.escape_identifier(CaseStyle::Pascal);
    let struct_name = format!("{}EncodedReturnValue", operation_name);
    // TODO: this should really be the parent interface (we just don't have access to it yet)
    let namespace = operation.namespace();

    // Should we assert instead?
    if !operation.has_encoded_result() {
        return "".into();
    }

    let parameters = operation.return_members(ast);

    let (dispatch_parameter, encoding) = if operation.returns_classes(ast) {
        (false, "IceRpc.Encoding.Ice11".to_owned())
    } else {
        (
            true,
            escape_parameter_name(&parameters, "dispatch") + ".GetIceEncoding()",
        )
    };

    let mut container_builder = ContainerBuilder::new(
        "public readonly record struct",
        &format!(
            "{}(global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>> Payload)",
            struct_name
        ),
    );

    container_builder.add_comment(
        "summary",
        &format!(
            "Helper record struct used to encode the return value of {} operation.",
            operation_name
        ),
    );

    let mut constructor_builder =
        FunctionBuilder::new("public", "", &struct_name, FunctionType::BlockBody);

    constructor_builder.add_comment(
        "summary",
        &format!(
            r#"Constructs a new <see cref="{struct_name}"/> instance that
immediately encodes the return value of operation {operation_name}."#,
            struct_name = struct_name,
            operation_name = operation_name
        ),
    );

    match operation.return_members(ast).as_slice() {
        [p] => {
            constructor_builder.add_parameter(
                &p.data_type
                    .to_type_string(&namespace, ast, TypeContext::Outgoing),
                "returnValue",
                None,
                None,
            );
        }
        _ => {
            for parameter in operation.return_members(ast) {
                let parameter_type =
                    parameter
                        .data_type
                        .to_type_string(&namespace, ast, TypeContext::Outgoing);
                let parameter_name = parameter.as_parameter_name("", true);

                constructor_builder.add_parameter(&parameter_type, &parameter_name, None, None);
            }
        }
    }

    if dispatch_parameter {
        constructor_builder.add_parameter(
            "IceRpc.Dispatch",
            &escape_parameter_name(&parameters, "dispatch"),
            None,
            None,
        );
    }

    constructor_builder.set_base_constructor("this");

    let create_payload = format!(
        "{encoding}.{method}(
    {return_value},
    {response_encode_action},
    classFormat: {class_format})",
        encoding = encoding,
        method = match parameters.as_slice() {
            [_] => "CreatePayloadFromSingleReturnValue",
            _ => "CreatePayloadFromReturnValueTuple",
        },
        return_value = if parameters.len() == 1 {
            "returnValue"
        } else {
            "returnValueTuple"
        },
        response_encode_action = response_encode_action(operation, ast),
        class_format = operation.format_type(),
    );

    constructor_builder.add_base_parameter(&create_payload);

    container_builder.add_block(constructor_builder.build());

    container_builder.build().into()
}
