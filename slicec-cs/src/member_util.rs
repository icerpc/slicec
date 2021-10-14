// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::{Ast, Node};
use slice::grammar::{Member, NamedSymbol, Primitive, ScopedSymbol};
use slice::util::{CaseStyle, TypeContext};

use crate::attributes::{custom_attributes, obsolete_attribute};
use crate::code_block::CodeBlock;
use crate::comments::{doc_comment_message, CommentTag};
use crate::cs_util::*;

pub fn to_argument_tuple(members: &[&Member], prefix: &str) -> String {
    match members {
        [] => panic!("tuple type with no members"),
        [member] => parameter_name(member, "", true),
        _ => format!(
            "({})",
            members
                .iter()
                .map(|m| parameter_name(m, prefix, true))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

pub fn to_tuple_type(members: &[&Member], scope: &str, ast: &Ast, context: TypeContext) -> String {
    match members {
        [] => panic!("tuple type with no members"),
        [member] => type_to_string(&member.data_type, scope, ast, context),
        _ => format!(
            "({})",
            members
                .iter()
                .map(|m| type_to_string(&m.data_type, scope, ast, context)
                    + " "
                    + &field_name(m, FieldType::NonMangled))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

pub fn field_name(member: &Member, field_type: FieldType) -> String {
    let identifier = escape_identifier(member, CaseStyle::Pascal);
    mangle_name(&identifier, field_type)
}

pub fn escape_parameter_name(parameters: &[&Member], name: &str) -> String {
    if parameters.iter().any(|p| p.identifier() == name) {
        name.to_owned() + "_"
    } else {
        name.to_owned()
    }
}

pub fn data_member_declaration(
    data_member: &Member,
    is_readonly: bool,
    field_type: FieldType,
    ast: &Ast,
) -> String {
    let data_type = &data_member.data_type;

    let type_string = type_to_string(data_type, data_member.scope(), ast, TypeContext::DataMember);
    let mut prelude = CodeBlock::new();

    prelude.writeln(&CommentTag::new(
        "summary",
        "",
        "",
        &doc_comment_message(data_member),
    ));
    prelude.writeln(
        &custom_attributes(data_member)
            .into_iter()
            .collect::<CodeBlock>(),
    );
    if let Some(obsolete) = obsolete_attribute(data_member, true) {
        prelude.writeln(&obsolete);
    }

    format!(
        "\
{prelude}
public {readonly}{type_string} {name};",
        prelude = prelude,
        readonly = if is_readonly { "readonly " } else { "" },
        type_string = type_string,
        name = field_name(data_member, field_type)
    )
}

pub fn is_member_default_initialized(member: &Member, ast: &Ast) -> bool {
    let data_type = &member.data_type;

    if data_type.is_optional {
        return true;
    }

    match data_type.definition(ast) {
        Node::Struct(_, struct_def) => struct_def
            .members(ast)
            .iter()
            .all(|m| is_member_default_initialized(m, ast)),
        _ => is_value_type(data_type, ast),
    }
}

pub fn initialize_non_nullable_fields(
    members: &[&Member],
    field_type: FieldType,
    ast: &Ast,
) -> CodeBlock {
    // This helper should only be used for classes and exceptions
    assert!(field_type == FieldType::Class || field_type == FieldType::Exception);

    let mut code = CodeBlock::new();

    for member in members {
        let data_type = &member.data_type;
        let data_node = data_type.definition(ast);
        if data_type.is_optional {
            continue;
        }

        let suppress = match data_node {
            Node::Class(_, _)
            | Node::Struct(_, _)
            | Node::Sequence(_, _)
            | Node::Dictionary(_, _) => true,
            Node::Primitive(_, primitive) if matches!(primitive, Primitive::String) => true,
            _ => false,
        };

        if suppress {
            // This is to suppress compiler warnings for non-nullable fields.
            writeln!(code, "this.{} = null!;", field_name(member, field_type));
        }
    }

    code
}
