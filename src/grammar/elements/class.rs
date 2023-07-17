// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub fields: Vec<WeakPtr<Field>>,
    pub compact_id: Option<Integer<u32>>,
    pub base: Option<TypeRef<Class>>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Class {
    pub fn fields(&self) -> Vec<&Field> {
        self.fields.iter().map(WeakPtr::borrow).collect()
    }

    pub fn all_inherited_fields(&self) -> Vec<&Field> {
        self.base_class().map(Class::fields).unwrap_or_default()
    }

    pub fn all_fields(&self) -> Vec<&Field> {
        let mut fields = vec![];
        // Recursively add inherited fields from super-classes.
        if let Some(base_class) = self.base_class() {
            fields.extend(base_class.all_fields());
        }
        fields.extend(self.fields());
        fields
    }

    pub fn base_class(&self) -> Option<&Class> {
        self.base.as_ref().map(|type_ref| type_ref.definition())
    }
}

impl Type for Class {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        true
    }

    fn tag_format(&self) -> Option<TagFormat> {
        Some(TagFormat::Class)
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Class, "class");
implement_Attributable_for!(Class);
implement_Entity_for!(Class);
implement_Commentable_for!(Class);
implement_Container_for!(Class, WeakPtr<Field>, fields);
