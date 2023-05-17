// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<WeakPtr<Enumerator>>,
    pub underlying: Option<TypeRef<Primitive>>,
    pub is_unchecked: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Enum {
    pub fn enumerators(&self) -> Vec<&Enumerator> {
        self.enumerators.iter().map(WeakPtr::borrow).collect()
    }

    pub fn get_min_max_values(&self) -> Option<(i128, i128)> {
        let values = self.enumerators.iter().map(|enumerator| enumerator.borrow().value());

        // There might not be a minimum value if the enum is empty.
        values.clone().min().map(|min| {
            (
                min,
                values.max().unwrap(), // A 'min' guarantees a 'max' exists too, so unwrap is safe.
            )
        })
    }
}

impl Type for Enum {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        self.underlying.as_ref().and_then(TypeRef::fixed_wire_size)
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        self.underlying.as_ref().map_or(
            Some(TagFormat::Size),              // Default value if `underlying` == None
            |data_type| data_type.tag_format(), // Expression to evaluate otherwise
        )
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Enum, "enum");
implement_Entity_for!(Enum);
implement_Commentable_for!(Enum);
implement_Container_for!(Enum, WeakPtr<Enumerator>, enumerators);
implement_Contained_for!(Enum, Module);
