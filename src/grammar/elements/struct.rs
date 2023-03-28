// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub fields: Vec<WeakPtr<Field>>,
    pub is_compact: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Struct {
    pub fn fields(&self) -> Vec<&Field> {
        self.fields.iter().map(WeakPtr::borrow).collect()
    }
}

impl Type for Struct {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        // Return `None` if any of the struct's fields aren't of fixed size.
        // Otherwise the fixed size of the struct is equal to the fixed size of its fields added together,
        // plus 1 if the struct isn't compact (to encode TagEndMarker).
        self.fields()
            .into_iter()
            .map(|field| field.data_type.fixed_wire_size())
            .collect::<Option<Vec<u32>>>() // ensure all fields are of fixed size; will return none if any are not
            .map(|sizes| sizes.iter().sum())
            .map(|size: u32| size + u32::from(!self.is_compact))
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.fixed_wire_size().is_some() {
            Some(TagFormat::VSize)
        } else {
            Some(TagFormat::FSize)
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Struct, "struct");
implement_Entity_for!(Struct);
implement_Container_for!(Struct, WeakPtr<Field>, fields);
implement_Contained_for!(Struct, Module);
