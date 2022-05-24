// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<Definition>,
    pub parent: Option<WeakPtr<Module>>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Module {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let contents = Vec::new();
        let parent = None;
        Module { identifier, contents, parent, scope, attributes, comment, location }
    }

    pub(crate) fn add_definition(&mut self, definition: Definition) {
        self.contents.push(definition);
    }

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
