// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: EnumeratorValue,
    pub associated_fields: Option<Vec<WeakPtr<Field>>>,
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

    pub fn associated_fields(&self) -> Option<Vec<&Field>> {
        self.associated_fields
            .as_ref()
            .map(|fields| fields.iter().map(WeakPtr::borrow).collect())
    }
}

#[derive(Debug)]
pub enum EnumeratorValue {
    Implicit(i128),
    Explicit(Integer<i128>),
}

impl Container<Field> for Enumerator {
    fn contents(&self) -> Vec<&Field> {
        self.associated_fields().unwrap_or_default()
    }
}

implement_Element_for!(Enumerator, "enumerator");
implement_Attributable_for!(@Contained Enumerator);
implement_Entity_for!(Enumerator);
implement_Commentable_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);
