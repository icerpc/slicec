// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_block::CodeBlock;
use crate::comments::*;
use crate::cs_util::*;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::ref_from_node;
use slice::util::*;
use slice::visitor::Visitor;
use slice::writer::Writer;

pub struct ProxyVisitor<'a> {
    output: &'a mut Writer,
}

impl<'a> ProxyVisitor<'a> {
    pub fn new(output: &'a mut Writer) -> ProxyVisitor<'a> {
        ProxyVisitor { output }
    }
}

impl Visitor for ProxyVisitor<'_> {
    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        // write_comment(&mut self.output, module_def);
        let content = format!("\nnamespace {}\n{{", module_def.identifier());
        self.output.write(&content);
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write("\n}");
        self.output.write_line_separator();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, ast: &Ast) {
        let prx_interface = format!("{}Prx", interface_name(interface_def)); // IFooPrx
        let prx_impl = prx_interface.chars().take(1); // IFooPrx -> FooPrx

        // writeProxyDocComment(p, getDeprecateReason(p));
        // emitCommonAttributes();
        // emitTypeIdAttribute(p->scoped());
        // emitCustomAttributes(p);
        // TODO: the above doc comments

        let bases: Vec<&Interface> = vec![];

        // Generate abstract methods and documentation
        write!(
            self.output,
            "
{doc_comment}
public partial interface {prx_interface}{prx_bases}
{{
    {operations}
}}",
            doc_comment = "foo",
            prx_interface = prx_interface,
            prx_bases = "",
            operations = prx_operations(interface_def, ast).indent()
        )
    }
}

pub fn interface_name(interface_def: &Interface) -> String {
    let identifier = fix_case(interface_def.identifier(), CaseStyle::Pascal);
    let mut chars = identifier.chars();

    // Check if the interface already follows the 'I' prefix convention.
    if identifier.chars().count() > 2
        && chars.next().unwrap() == 'I'
        && chars.next().unwrap().is_uppercase()
    {
        identifier.to_owned()
    } else {
        format!("I{}", identifier)
    }
}

pub fn prx_operations(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let operations = interface_def.operations(ast);

    for operation in operations {
        let operation_name = escape_identifier(operation, CaseStyle::Pascal);
        let async_name = operation_name + "Async";

        writeln!(
            code,
            "{doc_comment}\n{return} {name}({params});\n",
            doc_comment = operation_doc_comment(operation, false, ast),
            return = operation_return_task(operation, false, ast),
            name = async_name,
            params = get_invocation_params(operation, ast).join(", ")
        )
    }

    code
}

pub fn operation_return_task(operation: &Operation, is_dispatch: bool, ast: &Ast) -> String {
    match operation.return_type {
        ReturnType::Void(_) => {
            if is_dispatch {
                "global::System.Threading.Tasks.ValueTask".to_owned()
            } else {
                "global::System.Threading.Tasks.Task".to_owned()
            }
        }
        _ => {
            let return_type = operation_return_type(operation, is_dispatch, ast);
            if is_dispatch {
                format!("global::System.Threading.Tasks.ValueTask<{}>", return_type)
            } else {
                format!("global::System.Threading.Tasks.Task<{}>", return_type)
            }
        }
    }
}

pub fn operation_return_type(operation: &Operation, is_dispatch: bool, ast: &Ast) -> String {
    let return_type = &operation.return_type;

    let has_marshaled_result = false; // TODO: do we still want to keep this?

    if is_dispatch && has_marshaled_result {
        return "".to_owned();
    }

    match return_type {
        ReturnType::Void(_) => "void".to_owned(),
        ReturnType::Single(type_ref, _) => param_type_to_string(&type_ref, is_dispatch, ast),
        ReturnType::Tuple(indices, _) => {
            let members = indices
                .iter()
                .map(|index| ref_from_node!(Node::Member, ast, *index))
                .collect::<Vec<&Member>>();
            to_tuple_type(&members, is_dispatch, ast)
        }
    }
}

pub fn to_tuple_type(members: &[&Member], is_dispatch: bool, ast: &Ast) -> String {
    if members.len() == 1 {
        return param_type_to_string(&members[0].data_type, is_dispatch, ast);
    } else {
        members
            .into_iter()
            .map(|m| param_type_to_string(&m.data_type, is_dispatch, ast))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

// TODO: maybe rename operation_param_to_string
pub fn param_type_to_string(type_ref: &TypeRef, is_dispatch: bool, ast: &Ast) -> String {
    let context = if is_dispatch {
        TypeContext::Incoming
    } else {
        TypeContext::Outgoing
    };

    type_to_string(type_ref, type_ref.scope(), ast, context)
}

pub fn get_invocation_params(operation: &Operation, ast: &Ast) -> Vec<String> {
    let mut params = Vec::new();

    let operation_parameters = operation.parameters(ast);

    for p in operation.parameters(ast) {
        params.push(format!(
            "{attributes}{param_type} {param_name}",
            attributes = "", // TOOD: getParamAttributes(p)
            param_type = type_to_string(&p.data_type, p.scope(), ast, TypeContext::Outgoing),
            param_name = parameter_name(p, "", true)
        ))
    }

    params.push(format!(
        "IceRpc.Invocation? {} = null",
        escape_member_name(&operation_parameters, "invocation")
    ));
    params.push(format!(
        "global::System.Threading.CancellationToken {} = default",
        escape_member_name(&operation_parameters, "cancel")
    ));

    params
}

pub fn parameter_name(parameter: &Member, prefix: &str, escape_keywords: bool) -> String {
    let name = prefix.to_owned() + &fix_case(&parameter.identifier(), CaseStyle::Pascal);

    if escape_keywords {
        escape_keyword(&name)
    } else {
        name
    }
}
