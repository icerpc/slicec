// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO split into SliceFile and Util files! No need to keep together!

use crate::cs_util::*;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::ref_from_node;
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

    /// Helper method that checks if a named symbol has a comment written on it, and if so, formats
    /// it as a C# style doc comment and writes it to the underlying output.
    fn write_comment(&mut self, named_symbol: &dyn NamedSymbol) {
        // If the symbol has a doc comment attached to it, write it's fields to the output.
        if let Some(comment) = &named_symbol.comment() {
            // Write the comment's summary message if it has one.
            if !comment.message.is_empty() {
                self.write_comment_field("summary", &comment.message, "");
            }

            // Write each of the comment's parameter fields.
            for param in &comment.params {
                let (identifier, description) = param;
                let attribute = format!(" name=\"{}\"", &identifier);
                self.write_comment_field("param", &description, &attribute);
            }

            // Write the comment's returns message if it has one.
            if let Some(returns) = &comment.returns {
                self.write_comment_field("returns", &returns, "");
            }

            // Write each of the comment's exception fields.
            for exception in &comment.throws {
                let (exception, description) = exception;
                let attribute = format!(" cref=\"{}\"", &exception);
                self.write_comment_field("exceptions", &description, &attribute);
            }
        }
    }

    fn write_comment_field(&mut self, field_name: &str, content: &str, attribute: &str) {
        let mut field_string = format!("/// <{}{}>", field_name, attribute);
        if !content.is_empty() {
            // Iterate through each line of the field's content, and at the end of each line, append a
            // newline followed by 3 forward slashes to continue the comment.
            for line in content.lines() {
                field_string += line;
                field_string += "\n/// ";
            }
            // Remove the trailing newline and slashes by truncating off the last 5 characters.
            field_string.truncate(field_string.len() - 5);
        }
        // Append a closing tag, and write the field.
        field_string = field_string + "</" + field_name + ">\n";
        self.output.write_all(&field_string);
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
        self.output.write_line_separator();
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
        self.output.write_line_separator();
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
        self.output.write_line_separator();
    }

    fn visit_operation_start(&mut self, operation: &Operation, _: usize, ast: &Ast) {
        self.write_comment(operation);
        let mut parameters_string = String::new();
        if !operation.parameters.is_empty() {
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
        }

        let content = format!(
            "\npublic {} {}({});",
            return_type_to_string(&operation.return_type, ast, TypeContext::Outgoing),
            operation.identifier(),
            parameters_string,
        );
        self.output.write_all(&content);
        self.output.write_line_separator();
    }

    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        self.write_comment(enum_def);
        let content = format!("\npublic enum {}", enum_def.identifier());
        self.output.write_all(&content);
        if let Some(underlying) = &enum_def.underlying {
            let node = ast.resolve_index(*underlying.definition.as_ref().unwrap());
            let underlying_type_string =
                format!(" : {}", type_to_string(node, ast, TypeContext::Nested));
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
        self.output.write_line_separator();
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
