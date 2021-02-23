
use crate::grammar::*;
use crate::ast::{SliceAst, SliceFile};

//------------------------------------------------------------------------------
// Visitor
//------------------------------------------------------------------------------
#[allow(unused_variables)]
pub trait Visitor {
    fn visit_file_start(&mut self, file: &SliceFile) {}
    fn visit_file_end(&mut self, file: &SliceFile) {}

    fn visit_module_start(&mut self, module_def: &Module) {}
    fn visit_module_end(&mut self, module_def: &Module) {}
    fn visit_struct_start(&mut self, struct_def: &Struct) {}
    fn visit_struct_end(&mut self, struct_def: &Struct) {}
    fn visit_interface_start(&mut self, interface_def: &Interface) {}
    fn visit_interface_end(&mut self, interface_def: &Interface) {}

    fn visit_data_member(&mut self, data_member: &DataMember) {}

    fn visit_identifier(&mut self, identifier: &Identifier) {}
    fn visit_type_use(&mut self, type_use: &TypeUse) {}
}

//------------------------------------------------------------------------------
// Visitable
//------------------------------------------------------------------------------
pub trait Visitable {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst);
}

impl Visitable for SliceFile {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_file_start(self);
        for id in self.definitions.iter() {
            ast.resolve_id(*id).visit(visitor, ast);
        }
        visitor.visit_file_end(self);
    }
}

impl Visitable for Module {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_module_start(self);
        for id in self.contents.iter() {
            ast.resolve_id(*id).visit(visitor, ast);
        }
        visitor.visit_module_end(self);
    }
}

impl Visitable for Struct {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_struct_start(self);
        for id in self.contents.iter() {
            ast.resolve_id(*id).visit(visitor, ast);
        }
        visitor.visit_struct_end(self);
    }
}

impl Visitable for Interface {
    fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_interface_start(self);
        visitor.visit_interface_end(self);
    }
}

impl Visitable for DataMember {
    fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_data_member(self);
    }
}

impl Visitable for Identifier {
    fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_identifier(self);
    }
}

impl Visitable for TypeUse {
    fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_type_use(self);
    }
}
