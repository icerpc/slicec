// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<Definition>,
    pub parent: Option<WeakPtr<Module>>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub span: Span,
}

impl Contained<Module> for Module {
    fn parent(&self) -> Option<&Module> {
        self.parent.as_ref().map(WeakPtr::borrow)
    }
}

implement_Element_for!(Module, "module");
implement_Entity_for!(Module);
implement_Container_for!(Module, Definition, contents);
