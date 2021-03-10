// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::cs_util::*;
use slice::ast::Ast;
use slice::visitor::Visitor;
use slice::grammar::*;
use slice::util::SliceFile;
use slice::writer::Writer;
use std::io;

pub struct CsWriter {
    output: Writer,
}

impl CsWriter {
    pub fn new(path: &str) -> io::Result<Self>{
        let output = Writer::new(&(path.to_owned() + ".cs"))?;
        Ok(CsWriter { output })
    }

    pub fn close(self) {
        self.output.close();
    }
}

impl Visitor for CsWriter {
    fn visit_file_start(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write_all("//Start of file\n".as_bytes());
    }

    fn visit_file_end(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write_all("//End of file\n".as_bytes());
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        let content = format!("namespace {}\n", module_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.write_all(b"{\n");
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, _: &Ast) {
        let content = format!("struct {}\n", struct_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.write_all(b"{\n");
        self.output.indent_by(4);
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, _: &Ast) {
        let content = format!("interface {}\n", interface_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.write_all(b"{\n");
        self.output.indent_by(4);
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_data_member(&mut self, data_member: &DataMember, _: usize, ast: &Ast) {
        let node = ast.resolve_index(*data_member.data_type.definition.as_ref().unwrap());
        let type_string = type_to_string(node);

        let content = format!("{} {};\n", type_string, data_member.identifier());
        self.output.write_all(content.as_bytes());
    }
}
