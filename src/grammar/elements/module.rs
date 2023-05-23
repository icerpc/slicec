// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<Definition>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub span: Span,
}

impl Attributable for Module {
    fn attributes(&self) -> Vec<&Attribute> {
        self.attributes.iter().map(WeakPtr::borrow).collect::<Vec<_>>()
    }

    fn all_attributes(&self) -> Vec<Vec<&Attribute>> {
        vec![self.attributes()]
    }
}

implement_Element_for!(Module, "module");
implement_Symbol_for!(Module);
implement_Named_Symbol_for!(Module);
implement_Scoped_Symbol_for!(Module);
implement_Container_for!(Module, Definition, contents);
impl Entity for Module {}
