// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::utils::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
}

impl Type for Dictionary {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        1
    }

    fn uses_classes(&self) -> bool {
        // It is disallowed for key types to use classes, so we only have to check the value type.
        self.value_type.uses_classes()
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.key_type.is_fixed_size() && self.value_type.is_fixed_size() {
            Some(TagFormat::VSize)
        } else {
            Some(TagFormat::FSize)
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        let mut encodings = self.key_type.supported_encodings();
        encodings.intersect_with(&self.value_type.supported_encodings());
        encodings
    }
}

implement_Element_for!(Dictionary, "dictionary");
