
use crate::ast::{Node, SliceAst};
use crate::error::ErrorHandler;
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
                    // There are no other other symbols that can appear in the lookup table.
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
pub(crate) struct TypePatcher;

impl TypePatcher {
    pub(crate) fn patch_types(ast: &mut SliceAst, lookup_table: &HashMap<String, usize>, error_handler: &mut ErrorHandler) {
        for node in ast.iter_mut() {
            let (scope, type_use) = match node {
                Node::DataMember(_, data_member) => {
                    (data_member.scope.as_ref().unwrap(), &mut data_member.data_type)
                },
                _ => { continue },
            };

            if type_use.definition.is_some() {
                continue;
            }

            match Self::find_type(scope, &type_use.type_name, lookup_table) {
                Some(index) => {
                    type_use.definition = Some(index);
                },
                None => {
                    error_handler.report_error((
                        format!("failed to resolve type `{}` in scope `{}`", &type_use.type_name, scope).as_str(),
                        type_use.location.clone(),
                    ).into());
                },
            }
        }
    }

    fn find_type(scope: &str, typename: &str, lookup_table: &HashMap<String, usize>) -> Option<usize> {
        // If the typename starts with '::' it's an absolute path, and we can directly look it up.
        if typename.starts_with("::") {
            return lookup_table.get(typename).map(|index| index.clone());
        }

        let parents: Vec<&str> = scope.split("::").collect();
        for i in (0..parents.len()).rev() {
            let test_scope = parents[..i].join("::") + "::" + typename;

            if let Some(result) = lookup_table.get(&test_scope) {
                return Some(result.clone());
            }
        }

        return None
    }
}
