// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<WeakPtr<Enumerator>>,
    pub underlying: Option<TypeRef<Primitive>>,
    pub is_unchecked: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Enum {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        identifier: Identifier,
        underlying: Option<TypeRef<Primitive>>,
        is_unchecked: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let enumerators = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Enum {
            identifier,
            enumerators,
            underlying,
            is_unchecked,
            parent,
            scope,
            attributes,
            comment,
            location,
            supported_encodings,
        }
    }

    pub(crate) fn add_enumerator(&mut self, enumerator: WeakPtr<Enumerator>) {
        self.enumerators.push(enumerator);
    }

    pub fn enumerators(&self) -> Vec<&Enumerator> {
        self.enumerators
            .iter()
            .map(|enumerator_ptr| enumerator_ptr.borrow())
            .collect()
    }

    pub fn get_min_max_values(&self) -> Option<(i64, i64)> {
        let values = self.enumerators.iter().map(|enumerator| enumerator.borrow().value);

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
    fn is_fixed_size(&self) -> bool {
        match &self.underlying {
            Some(underlying) => underlying.is_fixed_size(),
            _ => false,
        }
    }

    fn min_wire_size(&self) -> u32 {
        match &self.underlying {
            Some(underlying) => underlying.min_wire_size(),
            _ => 1,
        }
    }

    fn uses_classes(&self) -> bool {
        false
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
implement_Container_for!(Enum, WeakPtr<Enumerator>, enumerators);
implement_Contained_for!(Enum, Module);
