
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

    fn resolve_id(&mut self, id: usize) -> &mut Box<dyn Node>;
}

//------------------------------------------------------------------------------
// Visitable
//------------------------------------------------------------------------------
pub trait Visitable {
    fn visit(&mut self, visitor: &mut dyn Visitor);
}

impl Visitable for Module {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_module(self);
        for id in self.contents().iter() {
            let content = visitor.resolve_id(*id);
            content.visit(visitor);
        }
    }
}

impl Visitable for Struct {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_struct(self);
        for data_member in self.contents().iter_mut() {
            data_member.visit(visitor);
        }
    }
}

impl Visitable for Interface {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_interface(self);
    }
}

impl Visitable for DataMember {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_data_member(self);
    }
}

impl Visitable for Identifier {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_identifier(self);
    }
}

impl Visitable for TypeUse {
    fn visit(&mut self, visitor: &mut dyn Visitor) {
        visitor.visit_type_use(self);
    }
}
