// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub fields: Vec<WeakPtr<Field>>,
    pub is_compact: bool,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Struct {
    pub fn fields(&self) -> Vec<&Field> {
        self.contents()
    }
}

impl Type for Struct {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }
}

implement_Element_for!(Struct, "struct");
implement_Attributable_for!(Struct);
implement_Entity_for!(Struct);
implement_Commentable_for!(Struct);
implement_Container_for!(Struct, Field, fields);
