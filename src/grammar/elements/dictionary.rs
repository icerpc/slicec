// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
}

impl Type for Dictionary {
    fn type_string(&self) -> String {
        format!(
            "dictionary<{}, {}>",
            self.key_type.type_string(),
            self.value_type.type_string(),
        )
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.key_type.fixed_wire_size().is_some() && self.value_type.fixed_wire_size().is_some() {
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
