// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<WeakPtr<Enumerator>>,
    pub underlying: Option<TypeRef<Primitive>>,
    pub is_compact: bool,
    pub is_unchecked: bool,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Enum {
    pub fn enumerators(&self) -> Vec<&Enumerator> {
        self.contents()
    }
}

impl Type for Enum {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }
}

implement_Element_for!(Enum, "enum");
implement_Attributable_for!(Enum);
implement_Entity_for!(Enum);
implement_Commentable_for!(Enum);
implement_Container_for!(Enum, Enumerator, enumerators);
