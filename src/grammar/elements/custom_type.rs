// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct CustomType {
    pub identifier: Identifier,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl CustomType {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        CustomType {
            identifier,
            parent,
            scope,
            attributes,
            comment,
            location,
            supported_encodings,
        }
    }
}

impl Type for CustomType {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        // TODO Can't we get rid of min wire size already?
        0
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        // Custom types are only supported with Slice2, which doesn't use tag formats.
        None
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(CustomType, "custom type");
implement_Entity_for!(CustomType);
implement_Contained_for!(CustomType, Module);
