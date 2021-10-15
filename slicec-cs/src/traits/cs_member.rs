// Copyright (c) ZeroC, Inc. All rights reserved.

use super::{CsNamedSymbol, CsTypeRef};
use crate::cs_util::{escape_keyword, mangle_name, FieldType};
use slice::ast::{Ast, Node};
use slice::grammar::{Member, NamedSymbol};
use slice::util::{fix_case, CaseStyle, TypeContext};

pub trait CsMemberInfo {
    fn as_parameter_name(&self, prefix: &str, escape_keywords: bool) -> String;
    fn field_name(&self, field_type: FieldType) -> String;
    fn is_default_initialized(&self, ast: &Ast) -> bool;
}

impl CsMemberInfo for Member {
    fn as_parameter_name(&self, prefix: &str, escape_keywords: bool) -> String {
        let name = prefix.to_owned() + &fix_case(self.identifier(), CaseStyle::Camel);

        if escape_keywords {
            escape_keyword(&name)
        } else {
            name
        }
    }

    fn field_name(&self, field_type: FieldType) -> String {
        mangle_name(&self.escape_identifier(CaseStyle::Pascal), field_type)
    }

    fn is_default_initialized(&self, ast: &Ast) -> bool {
        let data_type = &self.data_type;

        if data_type.is_optional {
            return true;
        }

        match data_type.definition(ast) {
            Node::Struct(_, struct_def) => struct_def
                .members(ast)
                .iter()
                .all(|m| m.is_default_initialized(ast)),
            _ => data_type.is_value_type(ast),
        }
    }
}

pub trait MemberListInfo {
    fn to_argument_tuple(&self, prefix: &str) -> String;
    fn to_tuple_type(&self, scope: &str, ast: &Ast, context: TypeContext) -> String;
    fn to_return_type(&self, scope: &str, ast: &Ast, context: TypeContext) -> String;
}

impl MemberListInfo for [&Member] {
    fn to_argument_tuple(&self, prefix: &str) -> String {
        match self {
            [] => panic!("tuple type with no members"),
            [member] => member.as_parameter_name("", true),
            _ => format!(
                "({})",
                self.iter()
                    .map(|m| m.as_parameter_name(prefix, true))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }

    fn to_tuple_type(&self, scope: &str, ast: &Ast, context: TypeContext) -> String {
        match self {
            [] => panic!("tuple type with no members"),
            [member] => member.data_type.type_to_string(scope, ast, context),
            _ => format!(
                "({})",
                self.iter()
                    .map(|m| m.data_type.type_to_string(scope, ast, context)
                        + " "
                        + &m.field_name(FieldType::NonMangled))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }

    fn to_return_type(&self, scope: &str, ast: &Ast, context: TypeContext) -> String {
        let value_task = "global::System.Threading.Tasks.ValueTask";
        match self {
            [] => value_task.to_owned(),
            [e] => {
                format!(
                    "{}<{}>",
                    value_task,
                    &e.data_type.type_to_string(scope, ast, context)
                )
            }
            _ => {
                format!(
                    "{}<({})>",
                    value_task,
                    self.iter()
                        .map(|e| {
                            format!(
                                "{} {}",
                                &e.data_type.type_to_string(scope, ast, context),
                                e.identifier()
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}
