// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::grammar::Operation;
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
