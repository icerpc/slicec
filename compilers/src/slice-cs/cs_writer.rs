
use slice::ast::Ast;
use slice::visitor::*;
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
}

impl Visitor for CsWriter {
    fn visit_file_start(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write_all("//Start of file\n".as_bytes());
    }

    fn visit_file_end(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write_all("//End of file\n".as_bytes());
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        let content = format!("namespace {}\n{{\n", module_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, _: &Ast) {
        let content = format!("struct {}\n{{\n", struct_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.indent_by(4);
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, _: &Ast) {
        let content = format!("interface {}\n{{\n", interface_def.identifier());
        self.output.write_all(content.as_bytes());
        self.output.indent_by(4);
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.output.indent_by(-4);
        let content = format!("}}\n");
        self.output.write_all(content.as_bytes());
    }

    fn visit_data_member(&mut self, data_member: &DataMember, _: usize, _: &Ast) {
        let content = format!("{} {};\n", data_member.data_type.type_name, data_member.identifier());
        self.output.write_all(content.as_bytes());
    }
}
