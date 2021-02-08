
use crate::definitions::*;

pub trait Visitor
{
    fn generate_from(&mut self, _: &Vec<Module>) {}

    fn visit_module(&mut self, _: &Module) {}
    fn visit_struct(&mut self, _: &Struct) {}
    fn visit_interface(&mut self, _: &Interface) {}
    fn visit_data_member(&mut self, _: &DataMember) {}
}



pub trait Visitable
{
    fn visit(&self, _: &mut dyn Visitor);
}

impl Visitable for Module
{
    fn visit(&self, visitor: &mut dyn Visitor)
    {
        visitor.visit_module(self);
    }
}

impl Visitable for Struct
{
    fn visit(&self, visitor: &mut dyn Visitor)
    {
        visitor.visit_struct(self);
    }
}

impl Visitable for Interface
{
    fn visit(&self, visitor: &mut dyn Visitor)
    {
        visitor.visit_interface(self);
    }
}

impl Visitable for DataMember
{
    fn visit(&self, visitor: &mut dyn Visitor)
    {
        visitor.visit_data_member(self);
    }
}