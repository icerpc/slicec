
use crate::definitions::*;
use crate::visitor::Visitor;
use crate::visitor::Visitable;

pub struct CsGenerator
{
    pub output: String,
}

impl Visitor for CsGenerator
{
    fn generate_from(&mut self, input: &Vec<Module>)
    {
        self.output.push_str("//\n// Copyright ZeroC (c)\n//\n\n");
        for module in input {
            module.visit(self);
        }
    }

    fn visit_module(&mut self, module_def: &Module)
    {
        let s = format!("namespace {}\n{{\n", module_def.identifier);
        self.output.push_str(&s);

        for definition in &module_def.content {
            (*definition).visit(self);
        }

        self.output.push_str("}\n\n");
    }

    fn visit_struct(&mut self, struct_def: &Struct)
    {
        let s = format!("public partial struct {}\n{{\n", struct_def.identifier);
        self.output.push_str(&s);

        for member in &struct_def.content {
            (*member).visit(self);
        }

        self.output.push_str("}\n\n");
    }

    fn visit_interface(&mut self, interface_def: &Interface)
    {
        let s = format!("public partial interface {}\n{{\n", interface_def.identifier);
        self.output.push_str(&s);

        self.output.push_str("}\n\n");
    }

    fn visit_data_member(&mut self, data_member_def: &DataMember)
    {
        let s = format!("public {} {};\n", data_member_def.typename.to_string(), data_member_def.identifier);
        self.output.push_str(&s);
    }

}
