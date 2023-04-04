// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Field {
    pub identifier: Identifier,
    pub data_type: TypeRef,
    pub tag: Option<Integer<u32>>,
    pub parent: WeakPtr<dyn Container<WeakPtr<Field>>>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

implement_Element_for!(Field, "field");
implement_Entity_for!(Field);
implement_Contained_for!(Field, dyn Container<WeakPtr<Field>> + 'static);
implement_Member_for!(Field);
