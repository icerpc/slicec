
use crate::ast::{Node, SliceAst};
use crate::error::ErrorHandler;
use crate::grammar::DataMember;
use crate::util::SliceFile;
use crate::visitor::Visitor;
use std::collections::HashMap;

//------------------------------------------------------------------------------
// ScopePatcher
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct ScopePatcher;

impl ScopePatcher {
    pub(crate) fn patch_scopes(ast: &mut SliceAst, lookup_table: &HashMap<String, usize>) {
        for (scoped_identifier, index) in lookup_table.iter() {
            // TODO replace this with 'https://github.com/rust-lang/rust/issues/74773' when it's merged into STABLE.
            let mut scope = scoped_identifier.rsplitn(2, "::").collect::<Vec<&str>>()[1].to_owned();
            if scope.is_empty() {
                scope = "::".to_owned();
            }

            let node = ast.resolve_index_mut(*index);
            match node {
                Node::Module(_, module_def) => {
                    module_def.scope = Some(scope);
                },
                Node::Struct(_, struct_def) => {
                    struct_def.scope = Some(scope);
                },
                Node::Interface(_, interface_def) => {
                    interface_def.scope = Some(scope);
                },
                Node::DataMember(_, data_member) => {
                    data_member.scope = Some(scope);
                },
                _ => {
                    panic!("Grammar element does not need scope patching!\n{:?}", node);
                }
            }
        }
    }
}

//------------------------------------------------------------------------------
// TypePatcher
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct TypePatcher<'a> {
    type_patches: Vec<(usize, usize)>,
    lookup_table: &'a HashMap<String, usize>,
    error_handler: &'a ErrorHandler,
}

impl<'a> TypePatcher<'a> {
    pub(crate) fn new(lookup_table: &'a HashMap<String, usize>, error_handler: &'a ErrorHandler) -> Self {
        TypePatcher {
            type_patches: Vec::new(),
            lookup_table,
            error_handler,
        }
    }

    pub(crate) fn patch_types(ast: &mut SliceAst,
                              slice_files: &HashMap<String, SliceFile>,
                              lookup_table: & HashMap<String, usize>,
                              error_handler: & ErrorHandler) {
        let mut type_patcher = TypePatcher::new(lookup_table, error_handler);

                                // TODO I don't think we really need this visitor here...
                                // we can just iterate over the nodes and directly check for DataMembers and set their definitions right there in place...
                                // but the visitor might make it easier? I don't know... I don't think so. Yeah, I think we shouldn't take the visitor approach here.

        for file in slice_files.values() {
            file.visit(&mut type_patcher, ast);
        }

        for (patch, index) in type_patcher.type_patches.into_iter() {
            let node = ast.resolve_index_mut(index);
            match node {
                Node::DataMember(_, data_member) => {
                    data_member.data_type.definition = Some(patch);
                },
                _ => {
                    panic!("Grammar element does not need type patching!\n{:?}", node);
                }
            }
        }
    }
}

impl<'a> Visitor for TypePatcher<'a> {
    fn visit_data_member(&mut self, data_member: &DataMember, index: usize) {

    }
}

//        let data_type = &data_member.data_type;
//        if data_type.definition.is_none() {
//            let type_name = data_type.type_name.as_str();//
//
//            // lookup the type somehow and resolve it!
//            self.type_patches.push((index, 54));
