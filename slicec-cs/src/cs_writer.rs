// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::cs_util::*;
use slice::ast::Ast;
use slice::grammar::*;
use slice::util::SliceFile;
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
        let content = format!("\nnamespace {}\n{{", module_def.identifier());
        self.output.write_all(content.as_str());
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, _: &Ast) {
        let content = format!("\nstruct {}\n{{", struct_def.identifier());
        self.output.write_all(content.as_str());
        self.output.indent_by(4);
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, _: &Ast) {
        let content = format!("\ninterface {}\n{{", interface_def.identifier());
        self.output.write_all(content.as_str());
        self.output.indent_by(4);
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write_all("\n}");
        self.output.write_line_seperator();
    }

    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        let content = format!("\npublic enum {}", enum_def.identifier());
        self.output.write_all(content.as_str());
        if let Some(underlying) = &enum_def.underlying {
            let node = ast.resolve_index(*underlying.definition.as_ref().unwrap());
            let underlying_type_string = format!(" : {}", type_to_string(node, ast));
            self.output.write_all(underlying_type_string.as_str());
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
        let content = format!("\n{} = {},", enumerator.identifier(), enumerator.value);
        self.output.write_all(content.as_str());
    }

    fn visit_data_member(&mut self, data_member: &DataMember, _: usize, ast: &Ast) {
        let node = ast.resolve_index(*data_member.data_type.definition.as_ref().unwrap());
        let type_string = type_to_string(node, ast);

        let content = format!("\n{} {};", type_string, data_member.identifier());
        self.output.write_all(content.as_str());
    }
}
