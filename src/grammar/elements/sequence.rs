// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
}

impl Sequence {
    pub fn has_fixed_size_numeric_elements(&self) -> bool {
        if self.element_type.is_optional {
            false
        } else {
            let mut definition = self.element_type.concrete_type();

            // If the elements are enums with an underlying type, check the underlying type instead.
            if let Types::Enum(enum_def) = definition {
                if let Some(underlying) = &enum_def.underlying {
                    definition = underlying.concrete_type();
                }
            }

            if let Types::Primitive(primitive) = definition {
                primitive.is_numeric_or_bool() && primitive.is_fixed_size()
            } else {
                false
            }
        }
    }
}

impl Type for Sequence {
    fn type_string(&self) -> String {
        format!("sequence<{}>", self.element_type.type_string())
    }

    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        1
    }

    fn uses_classes(&self) -> bool {
        self.element_type.uses_classes()
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.element_type.is_fixed_size() {
            if self.element_type.min_wire_size() == 1 {
                Some(TagFormat::OvSize)
            } else {
                Some(TagFormat::VSize)
            }
        } else {
            Some(TagFormat::FSize)
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.element_type.supported_encodings()
    }
}

implement_Element_for!(Sequence, "sequence");
