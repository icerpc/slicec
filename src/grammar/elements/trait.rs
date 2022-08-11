// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Trait {
    pub identifier: Identifier,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Trait {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        span: Span,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Trait {
            identifier,
            parent,
            scope,
            attributes,
            comment,
            span,
            supported_encodings,
        }
    }
}

impl Type for Trait {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        // Traits are encoded as a type-id string followed by a concrete type, encoded normally.
        // The min wire size is an empty type-id (1 byte) and an empty concrete type.
        1
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        // Traits are only supported with Slice2, which doesn't use tag formats.
        None
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Trait, "trait");
implement_Entity_for!(Trait);
implement_Contained_for!(Trait, Module);
