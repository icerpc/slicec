
use crate::grammar::*;

//------------------------------------------------------------------------------
// Visitor
//------------------------------------------------------------------------------
pub trait Visitor {
    fn visit_module(&mut self, module_def: &mut Module) {}
    fn visit_struct(&mut self, struct_def: &mut Struct) {}
    fn visit_interface(&mut self, interface_def: &mut Interface) {}
    fn visit_data_member(&mut self, data_member: &mut DataMember) {}
    fn visit_identifier(&mut self, identifier: &mut Identifier) {}
    fn visit_type_use(&mut self, type_use: &mut TypeUse) {}

    fn resolve_id(&mut self, id: usize) -> &mut dyn Node;
}

//------------------------------------------------------------------------------
// Visitable
//------------------------------------------------------------------------------
pub trait Visitable : Node {
    fn visit(&mut self, visitor: &mut dyn Visitor);
}

impl Visitable for Module {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_module(&mut self);
        for content in self.contents().iter_mut() {
            (self.resolve_id(content)).visit(visitor);
        }
    }
}

impl Visitable for Struct {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_struct(&mut self);
        for content in self.contents().iter_mut() {
            (self.resolve_id(content)).visit(visitor);
        }
    }
}

impl Visitable for Interface {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_interface(&mut self);
    }
}

impl Visitable for DataMember {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_data_member(&mut self);
    }
}

impl Visitable for Identifier {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_identifier(&mut self);
    }
}

impl Visitable for TypeUse {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_type_use(&mut self);
    }
}
