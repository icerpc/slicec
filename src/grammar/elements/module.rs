// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<Definition>,
    pub is_file_scoped: bool,
    pub parent: Option<WeakPtr<Module>>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Module {
    pub fn is_top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn submodules(&self) -> Vec<&Module> {
        self.contents
            .iter()
            .filter_map(|definition| {
                if let Definition::Module(module_def) = definition {
                    Some(module_def.borrow())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Contained<Module> for Module {
    fn parent(&self) -> Option<&Module> {
        self.parent.as_ref().map(|ptr| ptr.borrow())
    }
}

implement_Element_for!(Module, "module");
implement_Entity_for!(Module);
implement_Container_for!(Module, Definition, contents);
