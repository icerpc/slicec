
use crate::grammar::*;
use crate::visitor::Visitor;
use std::collections::HashMap;

//------------------------------------------------------------------------------
// TableBuilder
//------------------------------------------------------------------------------
pub(crate) struct TableBuilder {
    lookup_table: HashMap<String, usize>,
    current_scope: Vec<String>, //TODO make this a vector of &'a str instead!
}

impl TableBuilder {
    pub(crate) fn new() -> Self {
        TableBuilder {
            lookup_table: HashMap::new(),
            current_scope: Vec::new(),
        }
    }

    pub(crate) fn into_table(self) -> HashMap<String, usize> {
        self.lookup_table
    }

    fn add_entry(&mut self, identifier: &str, index: usize) {
        let scoped_identifier = self.current_scope.join("::") + "::" + identifier;
        self.lookup_table.insert(scoped_identifier, index);
    }
}

impl Visitor for TableBuilder {
    fn visit_module_start(&mut self, module_def: &Module, index: usize) {
        self.add_entry(module_def.identifier(), index);
        self.current_scope.push(module_def.identifier().to_owned());
    }

    fn visit_module_end(&mut self, _: &Module, _: usize) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, index: usize) {
        self.add_entry(struct_def.identifier(), index);
        self.current_scope.push(struct_def.identifier().to_owned());
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize) {
        self.current_scope.pop();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, index: usize) {
        self.add_entry(interface_def.identifier(), index);
        self.current_scope.push(interface_def.identifier().to_owned());
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize) {
        self.current_scope.pop();
    }

    fn visit_data_member(&mut self, data_member: &DataMember, index: usize) {
        self.add_entry(data_member.identifier(), index);
    }
}
