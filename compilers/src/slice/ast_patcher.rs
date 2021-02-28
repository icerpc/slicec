
use crate::ast::{Node, SliceAst};
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::visitor::Visitor;
use std::collections::HashMap;

//------------------------------------------------------------------------------
// AstPatcher
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct AstPatcher<'a> {
    type_patches: Vec<(usize, usize)>,
    lookup_table: &'a HashMap<String, usize>,
    error_handler: &'a ErrorHandler,
}

impl<'a> AstPatcher<'a> {
    pub(crate) fn new(lookup_table: &'a HashMap<String, usize>, error_handler: &'a ErrorHandler) -> Self {
        AstPatcher {
            type_patches: Vec::new(),
            lookup_table,
            error_handler,
        }
    }

    pub(crate) fn commit_patches(self, ast: &mut SliceAst) {
        for (index, patch) in self.type_patches.into_iter() {
            let node = ast.resolve_index_mut(index);
            match node {
                Node::DataMember(_, data_member) => {
                    data_member.data_type.definition = Some(patch);
                },
                _ => {
                    panic!("Grammar element does not need patching!\n{:?}", node);
                }
            }
        }
    }
}

//TODO should we patch other data too? Other than just the types?
impl<'a> Visitor for AstPatcher<'a> {
    fn visit_data_member(&mut self, data_member: &DataMember, index: usize) {
        let data_type = &data_member.data_type;
        if data_type.definition.is_none() {




            // lookup the type somehow and resolve it!
            self.type_patches.push((index, 54));
        }
    }
}
