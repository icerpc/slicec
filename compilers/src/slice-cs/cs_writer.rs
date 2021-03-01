
// TODO we need to write a single 'writer.rs' implementation in libslice that these can then use.

use slice::visitor::*;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use slice::grammar::*;
use slice::util::SliceFile;

pub struct CsWriter {
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
    fn visit_file_start(&mut self, _: &SliceFile) {
        self.file_buffer.write("//Start of file\n".as_bytes()).unwrap();
    }

    fn visit_file_end(&mut self, _: &SliceFile) {
        self.file_buffer.write("//End of file\n".as_bytes()).unwrap();
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize) {
        let content = format!("namespace {}\n{{\n", module_def.identifier());
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_module_end(&mut self, _: &Module, _: usize) {
        self.file_buffer.write("}\n".as_bytes()).unwrap();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize) {
        let content = format!("struct {}\n{{\n", struct_def.identifier());
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize) {
        self.file_buffer.write("}\n".as_bytes()).unwrap();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize) {
        let content = format!("interface {}\n{{\n", interface_def.identifier());
        self.file_buffer.write(content.as_bytes()).unwrap();
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize) {
        self.file_buffer.write("}\n".as_bytes()).unwrap();
    }

    fn visit_data_member(&mut self, data_member: &DataMember, _: usize) {
        let content = format!("{} {};\n", data_member.data_type.type_name, data_member.identifier());
        self.file_buffer.write(content.as_bytes()).unwrap();
    }
}
