// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: i128,
    pub parent: WeakPtr<Enum>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

implement_Element_for!(Enumerator, "enumerator");
implement_Entity_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);
