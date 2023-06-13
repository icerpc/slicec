// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Attribute {
    pub kind: Box<dyn AttributeKind>,
    pub span: Span,
}

impl Attribute {
    pub fn new(directive: String, args: Vec<String>, span: Span) -> Self {
        let kind = Box::new(attributes::Unparsed { directive, args });
        Self { kind, span }
    }

    pub fn downcast<T: AttributeKind + 'static>(&self) -> Option<&T> {
        self.kind.as_any().downcast_ref()
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
