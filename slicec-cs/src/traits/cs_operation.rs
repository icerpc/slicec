// Copyright (c) ZeroC, Inc. All rights reserved.

use super::{CsNamedSymbol, CsTypeRef, MemberListInfo};
use slice::ast::Ast;
use slice::grammar::{Operation, ScopedSymbol};
use slice::util::{CaseStyle, TypeContext};

pub trait CsOperation {
    fn has_encoded_result(&self) -> bool;
    fn encoded_result_struct(&self, scope: &str) -> String;
    fn format_type(&self) -> String;
    fn return_task(&self, scope: &str, is_dispatch: bool, ast: &Ast) -> String;
}

impl CsOperation for Operation {
    fn format_type(&self) -> String {
        // TODO: Austin - Implement this :)
        "default".to_owned()
    }

    // TODO: should this move to slice library that and take a language prefix parameter?
    // parameter
    fn has_encoded_result(&self) -> bool {
        self.has_attribute("cs:encoded-result")
        // || interface_def.has_attribute("cs:encoded-result")
        // TODO: also check the operation's parent interface once we can access it
    }

    fn encoded_result_struct(&self, scope: &str) -> String {
        self.escape_scoped_identifier(CaseStyle::Pascal, scope) + "EncodedReturnValue"
    }

    fn return_task(&self, scope: &str, is_dispatch: bool, ast: &Ast) -> String {
        let return_members = self.return_members(ast);
        if return_members.is_empty() {
            if is_dispatch {
                "global::System.Threading.Tasks.ValueTask".to_owned()
            } else {
                "global::System.Threading.Tasks.Task".to_owned()
            }
        } else {
            let return_type = operation_return_type(
                self,
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
}

fn operation_return_type(
    operation: &Operation,
    scope: &str,
    is_dispatch: bool,
    ast: &Ast,
    context: TypeContext,
) -> String {
    let return_members = operation.return_members(ast);

    if !return_members.is_empty() && is_dispatch && operation.has_encoded_result() {
        return operation.encoded_result_struct(scope);
    }

    match return_members.as_slice() {
        [] => "void".to_owned(),
        [member] => member.data_type.type_to_string(scope, ast, context),
        _ => return_members.to_tuple_type(scope, ast, context),
    }
}
