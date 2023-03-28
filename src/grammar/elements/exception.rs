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
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Exception {
    pub fn fields(&self) -> Vec<&Field> {
        self.fields.iter().map(WeakPtr::borrow).collect()
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

impl Type for Exception {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        // Return `None` if any of the exception's fields aren't of fixed size.
        // Otherwise the fixed size of the exception is equal to the fixed size of its fields added together.
        self.all_fields()
            .into_iter()
            .map(|field| field.data_type.fixed_wire_size())
            .collect::<Option<Vec<u32>>>() // ensure all fields are of fixed size; will return none if any are not
            .map(|sizes| sizes.iter().sum())
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        // Exceptions as a data type are only supported with Slice2, which doesn't use tag formats.
        None
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Exception, "exception");
implement_Entity_for!(Exception);
implement_Container_for!(Exception, WeakPtr<Field>, fields);
implement_Contained_for!(Exception, Module);
