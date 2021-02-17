
use crate::grammar::*;
use crate::parser::{SliceAst, SliceFile};

//------------------------------------------------------------------------------
// Visitor
//------------------------------------------------------------------------------
pub trait Visitor {
    fn visit_file_start(&mut self, file: &SliceFile,  ast: &mut SliceAst) {}
    fn visit_file_end(&mut self, file: &SliceFile,  ast: &mut SliceAst) {}
    fn visit_module_start(&mut self, module_def: &mut Module, ast: &mut SliceAst) {}
    fn visit_module_end(&mut self, module_def: &mut Module, ast: &mut SliceAst) {}
    fn visit_struct_start(&mut self, struct_def: &mut Struct, ast: &mut SliceAst) {}
    fn visit_struct_end(&mut self, struct_def: &mut Struct, ast: &mut SliceAst) {}
    fn visit_interface_start(&mut self, interface_def: &mut Interface, ast: &mut SliceAst) {}
    fn visit_interface_end(&mut self, interface_def: &mut Interface, ast: &mut SliceAst) {}
    fn visit_data_member(&mut self, data_member: &mut DataMember, ast: &mut SliceAst) {}
    fn visit_identifier(&mut self, identifier: &mut Identifier, ast: &mut SliceAst) {}
    fn visit_type_use(&mut self, type_use: &mut TypeUse, ast: &mut SliceAst) {}
}

//------------------------------------------------------------------------------
// Visitable
//------------------------------------------------------------------------------
pub trait Visitable {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst);
}

impl Visitable for SliceFile {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_file_start(self, ast);
        for id in self.contents.iter() {
            let mut content = ast.resolve_id(*id);
            content.visit(visitor, ast);
        }
        visitor.visit_file_end(self, ast);
    }
}

impl Visitable for Module {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_module_start(self, ast);
        for id in self.contents.iter() {
            let mut content = ast.resolve_id(*id);
            content.visit(visitor, ast);
        }
        visitor.visit_module_end(self, ast);
    }
}

impl Visitable for Struct {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_struct_start(self, ast);
        for data_member in self.contents.iter_mut() {
            data_member.visit(visitor, ast);
        }
        visitor.visit_struct_end(self, ast);
    }
}

impl Visitable for Interface {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_interface_start(self, ast);
        visitor.visit_interface_end(self, ast);
    }
}

impl Visitable for DataMember {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_data_member(self, ast);
    }
}

impl Visitable for Identifier {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_identifier(self, ast);
    }
}

impl Visitable for TypeUse {
    fn visit(&mut self, visitor: &mut dyn Visitor, ast: &mut SliceAst) {
        visitor.visit_type_use(self, ast);
    }
}
