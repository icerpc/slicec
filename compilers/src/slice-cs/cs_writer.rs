
// TODO we need to write a single 'writer.rs' implementation in libslice that these can then use.

use slice::ast::Ast;
use slice::visitor::*;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use slice::grammar::*;
use slice::util::SliceFile;

pub struct CsWriter {
    indent: usize,
    file_buffer: BufWriter<File>,
}

impl CsWriter {
    pub fn new(path: &str) -> Self {
        let file_path = path.to_owned() + ".cs";
        let file = match File::create(&file_path) {
            Ok(file) => file,
            Err(_) => { panic!("whatever"); },
        };

        Self {
            indent: 0,
            file_buffer: BufWriter::new(file),
        }
    }

    pub fn flush(&mut self) {
        match self.file_buffer.flush() {
            Ok(()) => {},
            Err(_) => { panic!("whatever2") },
        };
    }
}

impl Visitor for CsWriter {
    fn visit_file_start(&mut self, _: &SliceFile, _: &Ast) {
        self.file_buffer.write("//Start of file\n".as_bytes()).unwrap();
    }

    fn visit_file_end(&mut self, _: &SliceFile, _: &Ast) {
        self.file_buffer.write("//End of file\n".as_bytes()).unwrap();
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        let content = format!("{:i$}namespace {}\n{{\n", "", module_def.identifier(), i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
        self.indent += 4;
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.indent -= 4;
        let content = format!("{:i$}}}\n", "", i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, _: &Ast) {
        let content = format!("{:i$}struct {}\n{{\n", "", struct_def.identifier(), i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
        self.indent += 4;
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.indent -= 4;
        let content = format!("{:i$}}}\n", "", i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, _: &Ast) {
        let content = format!("{:i$}interface {}\n{{\n", "", interface_def.identifier(), i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
        self.indent += 4;
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.indent -= 4;
        let content = format!("{:i$}}}\n", "", i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_data_member(&mut self, data_member: &DataMember, _: usize, _: &Ast) {
        let content = format!("{:i$}{} {};\n", "", data_member.data_type.type_name, data_member.identifier(), i=self.indent);
        self.file_buffer.write(content.as_bytes()).unwrap();
    }
}
