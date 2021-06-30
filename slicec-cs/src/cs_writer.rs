// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO split into SliceFile and Util files! No need to keep together!

use crate::cs_util::*;
use slice::ref_from_node;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::{SliceFile, TypeContext};
use slice::visitor::Visitor;
use slice::writer::Writer;
use std::io;

pub struct CsWriter {
    output: Writer,
}

impl CsWriter {
    pub fn new(path: &str) -> io::Result<Self> {
        let output = Writer::new(&(path.to_owned() + ".cs"))?;
        Ok(CsWriter { output })
    }

    pub fn close(self) {
        self.output.close();
    }

    fn write_comment(&mut self, named_symbol: &dyn NamedSymbol) {
        if let Some(comment) = &named_symbol.comment() {
            if !comment.message.is_empty() {
                let mut summary_string = "/// <summary>".to_owned();
                // Iterate through each line of the comment, and at the end of each line, append a
                // newline followed by 3 forward slashes to continue the comment.
                for line in comment.message.lines() {
                    summary_string += line;
                    summary_string += "\n/// ";
                }
                // Remove the trailing newline and slashes, and append a closing summary tag.
                summary_string.truncate(summary_string.len() - 5);
                summary_string += "</summary>\n";
                self.output.write_all(&summary_string);
            }

            for param in &comment.params {
                let (identifier, description) = param;
                let mut param_string = format!("/// <param name=\"{}\">", identifier);
                if !description.is_empty() {
                    // Iterate through each line of the description, and at the end of each line,
                    // append a newline followed by 3 forward slashes to continue the comment.
                    for line in description.lines() {
                        param_string += line;
                        param_string += "\n/// ";
                    }
                    // Remove the trailing newline and slashes
                    param_string.truncate(param_string.len() - 5);
                }
                param_string += "</param>\n";
                self.output.write_all(&param_string);
            }

            if let Some(returns) = &comment.returns {
                let mut returns_string = "/// <returns>".to_owned();
                // Iterate through each line of the return message, and at the end of each line,
                // append a newline followed by 3 forward slashes to continue the comment.
                for line in returns.lines() {
                    returns_string += line;
                    returns_string += "\n/// ";
                }
                // Remove the trailing newline and slashes, and append a closing returns tag.
                returns_string.truncate(returns_string.len() - 5);
                returns_string += "</returns>\n";
                self.output.write_all(&returns_string);
            }

            for exception in &comment.throws {
                let (exception, description) = exception;
                let mut throws_string = format!("/// <exceptions cref=\"{}\">", exception);
                if !description.is_empty() {
                    // Iterate through each line of the description, and at the end of each line,
                    // append a newline followed by 3 forward slashes to continue the comment.
                    for line in description.lines() {
                        throws_string += line;
                        throws_string += "\n/// ";
                    }
                    // Remove the trailing newline and slashes
                    throws_string.truncate(throws_string.len() - 5);
                }
                throws_string += "</exceptions>\n";
                self.output.write_all(&throws_string);
            }
        }
    }
}

impl Visitor for CsWriter {
    fn visit_file_start(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write_all("//Start of file\n");
    }

    fn visit_file_end(&mut self, _: &SliceFile, _: &Ast) {
        self.output.clear_line_separator();
        self.output.write_all("\n//End of file\n");
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        self.write_comment(module_def);
        let content = format!("\nnamespace {}\n{{", module_def.identifier());
        self.output.write_all(&content);
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, _: &Ast) {
        self.write_comment(struct_def);
        let content = format!("\nstruct {}\n{{", struct_def.identifier());
        self.output.write_all(&content);
        self.output.indent_by(4);
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, _: &Ast) {
        self.write_comment(interface_def);
        let content = format!("\ninterface {}\n{{", interface_def.identifier());
        self.output.write_all(&content);
        self.output.indent_by(4);
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_operation_start(&mut self, operation: &Operation, _: usize, ast: &Ast) {
        self.write_comment(operation);
        let mut parameters_string = String::new();
        for id in operation.parameters.iter() {
            let parameter = ref_from_node!(Node::Member, ast, *id);
            let data_type = ast.resolve_index(parameter.data_type.definition.unwrap());
            parameters_string += format!(
                "{} {}, ",
                type_to_string(data_type, ast, TypeContext::Outgoing),
                parameter.identifier(),
            ).as_str();
        }
        // Remove the trailing comma and space.
        parameters_string.truncate(parameters_string.len() - 2);

        let content = format!(
            "\npublic {} {}({});",
            return_type_to_string(&operation.return_type, ast, TypeContext::Outgoing),
            operation.identifier(),
            parameters_string,
        );
        self.output.write_all(&content);
        self.output.write_line_seperator();
    }

    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        self.write_comment(enum_def);
        let content = format!("\npublic enum {}", enum_def.identifier());
        self.output.write_all(&content);
        if let Some(underlying) = &enum_def.underlying {
            let node = ast.resolve_index(*underlying.definition.as_ref().unwrap());
            let underlying_type_string = format!(
                " : {}",
                type_to_string(node, ast, TypeContext::Nested),
            );
            self.output.write_all(&underlying_type_string);
        } else {
            self.output.write_all(" : int")
        }
        self.output.write_all("\n{");
        self.output.indent_by(4);
    }

    fn visit_enum_end(&mut self, _: &Enum, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator, _: usize, _: &Ast) {
        self.write_comment(enumerator);
        let content = format!("\n{} = {},", enumerator.identifier(), enumerator.value);
        self.output.write_all(&content);
    }

    fn visit_data_member(&mut self, data_member: &Member, _: usize, ast: &Ast) {
        self.write_comment(data_member);
        let node = ast.resolve_index(*data_member.data_type.definition.as_ref().unwrap());
        let type_string = type_to_string(node, ast, TypeContext::DataMember);

        let content = format!("\n{} {};", type_string, data_member.identifier());
        self.output.write_all(&content);
    }
}
