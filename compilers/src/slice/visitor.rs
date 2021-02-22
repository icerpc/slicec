
use crate::grammar::*;
use crate::ast::{SliceAst, SliceFile};

//TODO maybe we don't need to provide the ast along with the visit functions?
//------------------------------------------------------------------------------
// Visitor
//------------------------------------------------------------------------------
#[allow(unused_variables)]
pub trait Visitor {
    fn visit_file_start(&mut self, file: &SliceFile,  ast: &SliceAst) {}
    fn visit_file_end(&mut self, file: &SliceFile,  ast: &SliceAst) {}

    fn visit_module_start(&mut self, module_def: &Module, ast: &SliceAst) {}
    fn visit_module_end(&mut self, module_def: &Module, ast: &SliceAst) {}
    fn visit_struct_start(&mut self, struct_def: &Struct, ast: &SliceAst) {}
    fn visit_struct_end(&mut self, struct_def: &Struct, ast: &SliceAst) {}
    fn visit_interface_start(&mut self, interface_def: &Interface, ast: &SliceAst) {}
    fn visit_interface_end(&mut self, interface_def: &Interface, ast: &SliceAst) {}

    fn visit_data_member(&mut self, data_member: &DataMember, ast: &SliceAst) {}

    fn visit_identifier(&mut self, identifier: &Identifier, ast: &SliceAst) {}
    fn visit_type_use(&mut self, type_use: &TypeUse, ast: &SliceAst) {}
}

//------------------------------------------------------------------------------
// Visitable
//------------------------------------------------------------------------------
pub trait Visitable {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst);
}

impl Visitable for SliceFile {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_file_start(self, ast);
        for id in self.definitions.iter() {
            ast.resolve_id(*id).visit(visitor, ast);
        }
        visitor.visit_file_end(self, ast);
    }
}

impl Visitable for Module {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_module_start(self, ast);
        for id in self.contents.iter() {
            ast.resolve_id(*id).visit(visitor, ast);
        }
        visitor.visit_module_end(self, ast);
    }
}

impl Visitable for Struct {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_struct_start(self, ast);
        for data_member in self.contents.iter() {
            data_member.visit(visitor, ast);
        }
        visitor.visit_struct_end(self, ast);
    }
}

impl Visitable for Interface {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_interface_start(self, ast);
        visitor.visit_interface_end(self, ast);
    }
}

impl Visitable for DataMember {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_data_member(self, ast);
    }
}

impl Visitable for Identifier {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_identifier(self, ast);
    }
}

impl Visitable for TypeUse {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_type_use(self, ast);
    }
}
