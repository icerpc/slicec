
use crate::grammar::*;
use crate::ast::{SliceAst, Node};
use crate::util::SliceFile;

//------------------------------------------------------------------------------
// Visitor
//------------------------------------------------------------------------------
/// Base trait for all visitors.
#[allow(unused_variables)] // We keep the parameter names for doc generation, even if unused in the default implementations.
pub trait Visitor {
    fn visit_file_start(&mut self, file: &SliceFile) {}
    fn visit_file_end(&mut self, file: &SliceFile) {}

    fn visit_module_start(&mut self, module_def: &Module, index: usize) {}
    fn visit_module_end(&mut self, module_def: &Module, index: usize) {}
    fn visit_struct_start(&mut self, struct_def: &Struct, index: usize) {}
    fn visit_struct_end(&mut self, struct_def: &Struct, index: usize) {}
    fn visit_interface_start(&mut self, interface_def: &Interface, index: usize) {}
    fn visit_interface_end(&mut self, interface_def: &Interface, index: usize) {}

    fn visit_data_member(&mut self, data_member: &DataMember, index: usize) {}

    fn visit_identifier(&mut self, identifier: &Identifier) {}
    fn visit_type_use(&mut self, type_use: &TypeUse) {}
}

//------------------------------------------------------------------------------
// Visit Functions
//------------------------------------------------------------------------------
impl Node {
    pub fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        // Forward the `visit` call to the underlying element.
        match self {
            Self::Module(index, module_def)       => { module_def.visit(visitor, ast, *index) },
            Self::Struct(index, struct_def)       => { struct_def.visit(visitor, ast, *index) },
            Self::Interface(index, interface_def) => { interface_def.visit(visitor, ast, *index) },
            Self::DataMember(index, data_member)  => { data_member.visit(visitor, ast, *index) },
            _ => { panic!("Cannot visit the following Node!\n{:?}", self) }
        }
    }
}

impl SliceFile {
    pub fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        visitor.visit_file_start(self);
        for id in self.contents.iter() {
            ast.resolve_index(*id).visit(visitor, ast);
        }
        visitor.visit_file_end(self);
    }
}

impl Module {
    pub fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst, index: usize) {
        visitor.visit_module_start(self, index);
        for id in self.contents.iter() {
            ast.resolve_index(*id).visit(visitor, ast);
        }
        visitor.visit_module_end(self, index);
    }
}

impl Struct {
    pub fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst, index: usize) {
        visitor.visit_struct_start(self, index);
        for id in self.contents.iter() {
            ast.resolve_index(*id).visit(visitor, ast);
        }
        visitor.visit_struct_end(self, index);
    }
}

impl Interface {
    pub fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst, index: usize) {
        visitor.visit_interface_start(self, index);
        visitor.visit_interface_end(self, index);
    }
}

impl DataMember {
    pub fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst, index: usize) {
        visitor.visit_data_member(self, index);
    }
}

impl Identifier {
    pub fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_identifier(self);
    }
}

impl TypeUse {
    pub fn visit(&self, visitor: &mut dyn Visitor, _: &SliceAst) {
        visitor.visit_type_use(self);
    }
}
