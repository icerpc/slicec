// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::grammar::{Operation, ScopedSymbol};
use slice::util::{CaseStyle, TypeContext};

use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::cs_util::{
    escape_identifier, escape_scoped_identifier, get_namespace, operation_format_type_to_string,
    parameter_name, type_to_string,
};
use crate::member_util::escape_parameter_name;

use crate::dispatch_visitor::response_encode_action;

// TODO: should this move to slice library that and take a language prefix parameter?
// parameter
pub fn has_encoded_result(operation_def: &Operation) -> bool {
    operation_def.has_attribute("cs:encoded-result")
    // || interface_def.has_attribute("cs:encoded-result")
    // TODO: also check the operation's parent interface once we can access it
}

pub fn encoded_result_struct_name(operation_def: &Operation, scope: &str) -> String {
    escape_scoped_identifier(operation_def, CaseStyle::Pascal, scope) + "EncodedReturnValue"
}

pub fn encoded_result_struct(operation: &Operation, ast: &Ast) -> CodeBlock {
    let operation_name = escape_identifier(operation, CaseStyle::Pascal);
    let struct_name = format!("{}EncodedReturnValue", operation_name);
    // TODO: this should really be the parent interface (we just don't have access to it yet)
    let namespace = get_namespace(operation);

    // Should we assert instead?
    if !has_encoded_result(operation) {
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

    let mut constructor_builder = FunctionBuilder::new("public", "", &struct_name);

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
                &type_to_string(&p.data_type, &namespace, ast, TypeContext::Outgoing),
                "returnValue",
                None,
                "",
            );
        }
        _ => {
            for parameter in operation.return_members(ast) {
                let parameter_type =
                    type_to_string(&parameter.data_type, &namespace, ast, TypeContext::Outgoing);
                let parameter_name = parameter_name(parameter, "", true);

                constructor_builder.add_parameter(&parameter_type, &parameter_name, None, "");
            }
        }
    }

    if dispatch_parameter {
        constructor_builder.add_parameter(
            "IceRpc.Dispatch",
            &escape_parameter_name(&parameters, "dispatch"),
            None,
            "",
        );
    }

    constructor_builder.use_this_base_constructor(true);

    let create_payload = format!(
        "\
{encoding}.{method}(
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
        class_format = operation_format_type_to_string(operation),
    );

    constructor_builder.add_base_argument(&create_payload);

    container_builder.add_block(constructor_builder.build());

    container_builder.build().into()
}
