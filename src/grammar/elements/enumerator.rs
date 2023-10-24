// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: EnumeratorValue,
    pub associated_fields: Vec<WeakPtr<Field>>,
    pub parent: WeakPtr<Enum>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Enumerator {
    pub fn value(&self) -> i128 {
        match &self.value {
            EnumeratorValue::Implicit(value) => *value,
            EnumeratorValue::Explicit(integer) => integer.value,
        }
    }

    pub fn associated_fields(&self) -> Vec<&Field> {
        self.contents()
    }
}

#[derive(Debug)]
pub enum EnumeratorValue {
    Implicit(i128),
    Explicit(Integer<i128>),
}

implement_Element_for!(Enumerator, "enumerator");
implement_Attributable_for!(@Contained Enumerator);
implement_Entity_for!(Enumerator);
implement_Commentable_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);
implement_Container_for!(Enumerator, Field, associated_fields);
