// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Exception {
    pub identifier: Identifier,
    pub fields: Vec<WeakPtr<Field>>,
    pub base: Option<TypeRef<Exception>>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Exception {
    pub fn fields(&self) -> Vec<&Field> {
        self.contents()
    }

    pub fn all_inherited_fields(&self) -> Vec<&Field> {
        self.base_exception().map(Exception::fields).unwrap_or_default()
    }

    pub fn all_fields(&self) -> Vec<&Field> {
        let mut fields = vec![];
        // Recursively add inherited fields from super-exceptions.
        if let Some(base_exception) = self.base_exception() {
            fields.extend(base_exception.all_fields());
        }
        fields.extend(self.fields());
        fields
    }

    pub fn base_exception(&self) -> Option<&Exception> {
        self.base.as_ref().map(TypeRef::definition)
    }
}

implement_Element_for!(Exception, "exception");
implement_Attributable_for!(Exception);
implement_Entity_for!(Exception);
implement_Commentable_for!(Exception);
implement_Container_for!(Exception, Field, fields);
