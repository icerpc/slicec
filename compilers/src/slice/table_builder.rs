
use crate::ast::Node;
use crate::grammar::*;
use crate::visitor::Visitor;

use std::collections::HashMap;

//------------------------------------------------------------------------------
// TableBuilder
//------------------------------------------------------------------------------
pub struct TableBuilder {
    lookup_table: HashMap<String, usize>,
    current_scope: Vec<String>,
}

impl TableBuilder {
    pub fn new() -> Self {
        TableBuilder {
            lookup_table: HashMap::new(),
            current_scope: Vec::new(),
        }
    }

    pub fn into_table(self) -> HashMap<String, usize> {
        self.lookup_table
    }

    fn add_entry(&mut self, identifier: &str, index: usize) {
        let scoped_identifier = self.current_scope.join("::") + "::" + identifier;
        self.lookup_table.insert(scoped_identifier, index);
    }
}

impl Visitor for TableBuilder {
    fn visit_module_start(&mut self, module_def: &Module) {
        self.add_entry(module_def.get_identifier(), module_def.index());
        self.current_scope.push(module_def.get_identifier().to_owned());
    }

    fn visit_module_end(&mut self, _: &Module) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.add_entry(struct_def.get_identifier(), struct_def.index());
        self.current_scope.push(struct_def.get_identifier().to_owned());
    }

    fn visit_struct_end(&mut self, _: &Struct) {
        self.current_scope.pop();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {
        self.add_entry(interface_def.get_identifier(), interface_def.index());
        self.current_scope.push(interface_def.get_identifier().to_owned());
    }

    fn visit_interface_end(&mut self, _: &Interface) {
        self.current_scope.pop();
    }

    fn visit_data_member(&mut self, data_member: &DataMember) {
        self.add_entry(data_member.get_identifier(), data_member.index());
    }
}
