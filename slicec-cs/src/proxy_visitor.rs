use slice::ast::{Ast, Node};
use slice::grammar::*;
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
}
