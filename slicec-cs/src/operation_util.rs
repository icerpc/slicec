// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::grammar::{Operation, ScopedSymbol};
use slice::util::TypeContext;

use crate::cs_util::*;
use crate::encoded_result::{encoded_result_struct_name, has_encoded_result};
use crate::member_util::*;

pub fn operation_return_task(
    operation: &Operation,
    scope: &str,
    is_dispatch: bool,
    ast: &Ast,
) -> String {
    let return_members = operation.return_members(ast);
    if return_members.is_empty() {
        if is_dispatch {
            "global::System.Threading.Tasks.ValueTask".to_owned()
        } else {
            "global::System.Threading.Tasks.Task".to_owned()
        }
    } else {
        let return_type = operation_return_type(
            operation,
            scope,
            is_dispatch,
            ast,
            if is_dispatch {
                TypeContext::Outgoing
            } else {
                TypeContext::Incoming
            },
        );
        if is_dispatch {
            format!("global::System.Threading.Tasks.ValueTask<{}>", return_type)
        } else {
            format!("global::System.Threading.Tasks.Task<{}>", return_type)
        }
    }
}

pub fn operation_return_type(
    operation: &Operation,
    scope: &str,
    is_dispatch: bool,
    ast: &Ast,
    context: TypeContext,
) -> String {
    let return_members = operation.return_members(ast);

    if !return_members.is_empty() && is_dispatch && has_encoded_result(operation) {
        return encoded_result_struct_name(operation, scope);
    }

    match return_members.as_slice() {
        [] => "void".to_owned(),
        [member] => type_to_string(&member.data_type, scope, ast, context),
        _ => to_tuple_type(&return_members, scope, ast, context),
    }
}

pub fn operation_params(operation: &Operation, is_dispatch: bool, ast: &Ast) -> Vec<String> {
    let mut params = Vec::new();

    let operation_parameters = operation.parameters(ast);

    for p in operation.parameters(ast) {
        params.push(format!(
            "{attributes}{param_type} {param_name}",
            attributes = "", // TOOD: getParamAttributes(p)
            param_type = type_to_string(
                &p.data_type,
                p.scope(),
                ast,
                if is_dispatch {
                    TypeContext::Incoming
                } else {
                    TypeContext::Outgoing
                }
            ),
            param_name = parameter_name(p, "", true)
        ))
    }

    params.push(if is_dispatch {
        format!(
            "IceRpc.Dispatch? {} = null",
            escape_parameter_name(&operation_parameters, "dispatch")
        )
    } else {
        format!(
            "IceRpc.Invocation? {} = null",
            escape_parameter_name(&operation_parameters, "invocation")
        )
    });
    params.push(format!(
        "global::System.Threading.CancellationToken {} = default",
        escape_parameter_name(&operation_parameters, "cancel")
    ));

    params
}
